import torch
from transformers import AutoTokenizer, AutoModelForCausalLM
import sys
import io

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def _input(prompt):
    print(prompt, end='', flush=True)
    s = sys.stdin.buffer.readline()
    s = s.decode('utf-8') \
        .replace("\r", "") \
        .replace("\n", "")
    return s

def make_prompt(log):
    prompt = [
        f"{uttr['speaker']}: {uttr['text']}"
        for uttr in log
    ]
    prompt = "<NL>".join(prompt)
    return prompt

def add_log(log, role, text):
    log.append({
        "speaker": role,
        "text": text
    })

k = 40

def gradually_generate(model, tokenizer, token_ids, max_length):
    for _ in range(max_length):
        with torch.no_grad():
            outputs = model(token_ids.to(model.device))

        logits = outputs.logits
        indices_to_remove = logits < torch.topk(logits, k)[0][..., -1, None]
        logits[indices_to_remove] = float('-inf')
        probs = torch.nn.functional.softmax(logits[..., -1, :], dim=-1)
        next_token_id = torch.multinomial(probs, num_samples=1)
        token_ids = torch.cat((token_ids, next_token_id), dim=-1)

        output_str = tokenizer.decode(next_token_id[0])

        yield output_str.replace("<NL>", "\n")

        if "</s>" in output_str:
            break

def llm_main(max_length = 128):
    tokenizer = AutoTokenizer.from_pretrained("rinna/japanese-gpt-neox-3.6b-instruction-sft", use_fast=False)
    model = AutoModelForCausalLM.from_pretrained("rinna/japanese-gpt-neox-3.6b-instruction-sft")

    log = []

    while "[exit]" not in (user_message := _input("> ")):

        # print(f"[[[user_message: '{user_message}']]]")

        add_log(log, "ユーザー", user_message)

        prompt = (
            make_prompt(log)
            + "<NL>"
            + "システム: "
        )

        token_ids = tokenizer.encode(prompt, add_special_tokens=False, return_tensors="pt")

        output = ""
        for word in gradually_generate(model, tokenizer, token_ids, max_length):
            word = word.replace(": ", "; ")
            print(word, end='', flush=True)
            output += word
        print()

        add_log(log, "システム", output)