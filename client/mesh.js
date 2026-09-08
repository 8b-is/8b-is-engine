// mesh.js — the browser's door to the actor-mesh, dependency-free.
//
// A minimal NATS client (core protocol) over WebSocket: CONNECT,
// PING/PONG, SUB/UNSUB, PUB, MSG — enough for zones = actor subjects.
// No npm, no bundler; an ES module the floors and the dashboard import
// directly. Works in browsers and in Node ≥ 22 (global WebSocket).
//
//   import { Mesh } from './mesh.js'
//   const mesh = await new Mesh('ws://127.0.0.1:9222').connect()
//   mesh.subscribe('gaia.state', (msg) => console.log(msg.data))
//   mesh.publish('actor.ལྷ.inbox', '{"s":0.2}')

export class Mesh {
  constructor(url = 'ws://127.0.0.1:9222') {
    this.url = url
    this.ws = null
    this.sid = 0
    this.inboxSeq = 0
    this.subs = new Map() // sid -> callback
    this.buf = ''         // unparsed protocol bytes
    this.pending = 0      // MSG payload bytes still to come
    this.pendingSid = ''
    this.pendingSubj = ''
    this.info = null
    this.lastError = null
    this.onclose = null
  }

  connect() {
    this.ready = new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.url)
      this.ws.binaryType = 'arraybuffer' // strings and bytes both arrive as buffers
      this.ws.onopen = () => {
        this._send('CONNECT {"verbose":false,"pedantic":false,"headers":false,"name":"mesh-js"}')
        this._send('PING')
        const timer = setTimeout(() => reject(new Error('nats handshake timeout')), 5000)
        this._pong = () => { clearTimeout(timer); resolve(this) }
      }
      this.ws.onmessage = (e) => this._onData(e.data)
      this.ws.onerror = () => reject(new Error('nats ws error'))
      this.ws.onclose = () => {
        for (const [, cb] of this.subs) { try { cb(null) } catch {} }
        if (this.onclose) this.onclose()
      }
    })
    return this.ready
  }

  close() {
    if (this.ws) this.ws.close()
  }

  _send(s) {
    // every protocol message is a CRLF-terminated line; PUB payloads
    // need the trailing CRLF too (the size field excludes it)
    if (this.ws && this.ws.readyState === 1) this.ws.send(s + '\r\n')
  }

  subscribe(subject, cb) {
    const sid = String(++this.sid)
    this.subs.set(sid, cb)
    this._send(`SUB ${subject} ${sid}`)
    return sid
  }

  unsubscribe(sid, max = 0) {
    this._send(`UNSUB ${sid}${max ? ` ${max}` : ''}`)
    this.subs.delete(sid)
  }

  publish(subject, payload) {
    const data = typeof payload === 'string' ? payload : JSON.stringify(payload)
    const size = new TextEncoder().encode(data).length
    this._send(`PUB ${subject} ${size}`)
    this._send(data)
  }

  // request/reply on the actor mesh: the reducer call from the browser.
  request(subject, payload, timeoutMs = 2000) {
    return new Promise((resolve, reject) => {
      const inbox = `_INBOX.meshjs.${this.sid}.${this.inboxSeq++}`
      const timer = setTimeout(() => { this.unsubscribe(sid); reject(new Error(`nats request timeout: ${subject}`)) }, timeoutMs)
      const sid = this.subscribe(inbox, (msg) => {
        clearTimeout(timer)
        this.unsubscribe(sid)
        resolve(msg ? msg.data : null)
      })
      const data = typeof payload === 'string' ? payload : JSON.stringify(payload)
      const size = new TextEncoder().encode(data).length
      this._send(`PUB ${subject} ${inbox} ${size}`)
      this._send(data)
    })
  }

  async _onData(data) {
    // string, ArrayBuffer, or (defensively) Blob
    let buf = data
    if (typeof Blob !== 'undefined' && data instanceof Blob) buf = await data.arrayBuffer()
    const text = typeof buf === 'string' ? buf : new TextDecoder().decode(buf)
    this.buf += text
    this._parse()
  }

  _parse() {
    // eslint-disable-next-line no-constant-condition
    while (true) {
      if (this.pending > 0) {
        if (this.buf.length < this.pending + 2) return
        const payload = this.buf.slice(0, this.pending)
        this.buf = this.buf.slice(this.pending + 2) // + trailing CRLF
        const cb = this.subs.get(this.pendingSid)
        this.pending = 0
        if (cb) cb({ subject: this.pendingSubj, data: payload })
        continue
      }
      const i = this.buf.indexOf('\r\n')
      if (i < 0) return
      const line = this.buf.slice(0, i)
      this.buf = this.buf.slice(i + 2)
      if (!line) continue
      const parts = line.split(' ')
      const op = parts[0]
      if (op === 'PING') {
        this._send('PONG')
      } else if (op === 'PONG') {
        if (this._pong) { this._pong(); this._pong = null }
      } else if (op === 'MSG') {
        // MSG <subject> <sid> [reply] <#bytes>
        this.pendingSubj = parts[1]
        this.pendingSid = parts[2]
        this.pending = Number(parts[parts.length - 1])
      } else if (op === '+OK' || op === '-ERR') {
        this.lastError = op === '-ERR' ? line.slice(5) : null
      } else if (op === 'INFO') {
        try { this.info = JSON.parse(line.slice(5)) } catch { /* keep null */ }
      }
    }
  }
}
