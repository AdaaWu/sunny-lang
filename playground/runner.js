// Sunny Playground Runner
// 支援兩種模式: WASM (生產) 和 JS 解譯器 (開發/Fallback)

let wasmModule = null;

async function initWasm() {
  try {
    const mod = await import('./pkg/sunny_lang.js');
    await mod.default();
    wasmModule = mod;
    console.log('Sunny WASM loaded');
    return true;
  } catch (e) {
    console.log('WASM not available, using JS interpreter fallback');
    return false;
  }
}

function runSunny(source) {
  if (wasmModule) {
    return runWasm(source);
  }
  return runFallback(source);
}

function runWasm(source) {
  try {
    const jsonStr = wasmModule.eval_sunny(source);
    return JSON.parse(jsonStr);
  } catch (e) {
    return { output: [], result: null, error: e.message };
  }
}

// JS Fallback 解譯器 (簡易版，用於 WASM 不可用時)
function runFallback(source) {
  const output = [];
  const errors = [];

  try {
    // 簡易 tokenizer + evaluator for basic demo
    const env = {};
    const lines = source.split('\n');

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('//')) continue;

      // print(...)
      const printMatch = trimmed.match(/^print\((.+)\)$/);
      if (printMatch) {
        const args = evalArgs(printMatch[1], env);
        output.push(args.join(' '));
        continue;
      }

      // lit name = value
      const litMatch = trimmed.match(/^lit\s+(\w+)\s*=\s*(.+)$/);
      if (litMatch) {
        env[litMatch[1]] = evalSimple(litMatch[2], env);
        continue;
      }

      // glow name = value
      const glowMatch = trimmed.match(/^glow\s+(\w+)\s*=\s*(.+)$/);
      if (glowMatch) {
        env[glowMatch[1]] = evalSimple(glowMatch[2], env);
        continue;
      }
    }
  } catch (e) {
    errors.push(e.message);
  }

  return {
    output,
    result: null,
    error: errors.length > 0 ? errors.join('\n') : null,
  };
}

function evalSimple(expr, env) {
  expr = expr.trim();
  // String
  if ((expr.startsWith('"') && expr.endsWith('"')) ||
      (expr.startsWith("'") && expr.endsWith("'"))) {
    return expr.slice(1, -1);
  }
  // Number
  if (/^-?\d+(\.\d+)?$/.test(expr)) {
    return expr.includes('.') ? parseFloat(expr) : parseInt(expr, 10);
  }
  // Bool
  if (expr === 'true') return true;
  if (expr === 'false') return false;
  // Variable
  if (env.hasOwnProperty(expr)) return env[expr];
  return expr;
}

function evalArgs(argsStr, env) {
  const results = [];
  let depth = 0;
  let current = '';
  for (const ch of argsStr) {
    if (ch === '(' || ch === '[') depth++;
    if (ch === ')' || ch === ']') depth--;
    if (ch === ',' && depth === 0) {
      results.push(String(evalSimple(current, env)));
      current = '';
    } else {
      current += ch;
    }
  }
  if (current.trim()) {
    results.push(String(evalSimple(current.trim(), env)));
  }
  return results;
}
