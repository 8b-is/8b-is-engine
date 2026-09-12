/* nand.c — the seed. NAND alone builds every logic gate; every computable
 * function is a composition of it (Turing's universality, one gate deep).
 * Pure: inputs in, truth out, no side effects.
 */
int nand(int a, int b) { return !(a && b); }

/* the universal gate, spelled out — NOT, AND, OR, XOR from NAND alone */
int not(int a)     { return nand(a, a); }
int and(int a, int b) { return not(nand(a, b)); }
int or(int a, int b)  { return nand(not(a), not(b)); }
int xor(int a, int b) { return and(or(a, b), nand(a, b)); }
