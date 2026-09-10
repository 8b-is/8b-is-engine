#!/usr/bin/env python3
"""run_finetune.py — the obliterated lane's headless training script.

Run against a Colab VM kernel via `colab exec -f run_finetune.py` (the
driver `drive.sh` orchestrates the whole chain): unsloth QLoRA at 4-bit
on the mix (the world set + a public instruct set), merged fp16 export,
GGUF Q4_K_M export, and MLX conversion inputs. Parameterized by env vars
so the same script drives any of the four picks:

    MODEL (default: OBLITERATUS/Ornith-1.5-9B-OBLITERATED)
    STEPS (default: 120) · WORLD (default: /content/world_dataset.jsonl)
"""

import os

MODEL = os.environ.get("MODEL", "OBLITERATUS/Ornith-1.5-9B-OBLITERATED")
STEPS = int(os.environ.get("STEPS", "120"))
WORLD = os.environ.get("WORLD", "/content/world_dataset.jsonl")
MAX_SEQ = 4096


def main():
    print(f"⟦ the obliterated lane ⟧ {MODEL} · {STEPS} steps · world {WORLD}", flush=True)

    # lane 0 · the tool
    import subprocess, sys

    subprocess.run(
        [sys.executable, "-m", "pip", "install", "-q", "unsloth[x]"], check=False
    )

    from unsloth import FastLanguageModel, UnslothTrainer, UnslothTrainingArguments

    # lane 1 · the 4-bit door
    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=MODEL,
        max_seq_length=MAX_SEQ,
        dtype=None,
        load_in_4bit=True,
    )
    model = FastLanguageModel.get_peft_model(
        model,
        r=16,
        target_modules=[
            "q_proj", "k_proj", "v_proj", "o_proj",
            "gate_proj", "up_proj", "down_proj",
        ],
    )

    # lane 2 · the mix
    from datasets import load_dataset

    world = load_dataset("json", data_files=WORLD, split="train")

    def fmt(ex):
        if "messages" in ex:
            return ex["messages"]
        return [{
            "role": "system",
            "content": ex.get("system", "you are the 8b-is engine."),
        }, {
            "role": "user",
            "content": ex.get("instruction", "the keeper folds."),
        }, {
            "role": "assistant",
            "content": ex.get("output", "admissible."),
        }]

    dataset = world.map(lambda ex: {"messages": fmt(ex)}).select_columns(["messages"])

    # lane 3 · the train
    trainer = UnslothTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=dataset,
        args=UnslothTrainingArguments(
            per_device_train_batch_size=2,
            gradient_accumulation_steps=4,
            max_steps=STEPS,
            learning_rate=2e-4,
            weight_decay=0.0,
            warmup_steps=10,
            lr_scheduler_type="linear",
            logging_steps=1,
            output_dir="/content/out/lora",
        ),
    )
    trainer.train()

    # lane 4 · the exports — the door to both formats
    FastLanguageModel.save_pretrained_merged(
        "/content/out/merged_fp16", tokenizer, save_method="merged_16bit"
    )
    FastLanguageModel.save_pretrained_gguf(
        "/content/out/gguf", tokenizer, quantization_method="q4_k_m"
    )
    print("⟦ done ⟧ /content/out is ready: lora · merged_fp16 · gguf/q4_k_m", flush=True)


if __name__ == "__main__":
    main()
