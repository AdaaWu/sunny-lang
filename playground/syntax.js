// Sunny 語法高亮（純 JS，不依賴 WASM）
const KEYWORDS = new Set([
  'lit', 'glow', 'fn', 'out', 'if', 'else', 'for', 'in', 'while',
  'match', 'is', 'ray', 'import', 'and', 'or', 'not', 'true', 'false'
]);

const TYPES = new Set([
  'Int', 'Float', 'String', 'Bool', 'List', 'Map', 'Shadow'
]);

const BUILTINS = new Set([
  'print', 'await', 'read_file', 'render_md', 'render_template',
  'write_file', 'len', 'type_of', 'to_int', 'to_float', 'to_string',
  'json_encode', 'time_now'
]);

function highlightSunny(code) {
  let result = '';
  let i = 0;
  const len = code.length;

  while (i < len) {
    // Comments
    if (code[i] === '/' && code[i + 1] === '/') {
      let end = code.indexOf('\n', i);
      if (end === -1) end = len;
      result += `<span class="highlight-comment">${escapeHtml(code.slice(i, end))}</span>`;
      i = end;
      continue;
    }
    if (code[i] === '/' && code[i + 1] === '*') {
      let end = code.indexOf('*/', i + 2);
      if (end === -1) end = len; else end += 2;
      result += `<span class="highlight-comment">${escapeHtml(code.slice(i, end))}</span>`;
      i = end;
      continue;
    }

    // Strings
    if (code[i] === '"' || code[i] === "'") {
      const quote = code[i];
      let j = i + 1;
      while (j < len && code[j] !== quote) {
        if (code[j] === '\\') j++;
        j++;
      }
      j++; // closing quote
      result += `<span class="highlight-string">${escapeHtml(code.slice(i, j))}</span>`;
      i = j;
      continue;
    }

    // Numbers
    if (/\d/.test(code[i])) {
      let j = i;
      while (j < len && /[\d.]/.test(code[j])) j++;
      result += `<span class="highlight-number">${code.slice(i, j)}</span>`;
      i = j;
      continue;
    }

    // Identifiers / Keywords
    if (/[a-zA-Z_]/.test(code[i])) {
      let j = i;
      while (j < len && /[a-zA-Z0-9_]/.test(code[j])) j++;
      const word = code.slice(i, j);
      if (KEYWORDS.has(word)) {
        result += `<span class="highlight-keyword">${word}</span>`;
      } else if (TYPES.has(word)) {
        result += `<span class="highlight-type">${word}</span>`;
      } else if (BUILTINS.has(word)) {
        result += `<span class="highlight-builtin">${word}</span>`;
      } else {
        result += escapeHtml(word);
      }
      i = j;
      continue;
    }

    // Operators
    if ('+-*/%=<>!&|.'.includes(code[i])) {
      result += `<span class="highlight-operator">${escapeHtml(code[i])}</span>`;
      i++;
      continue;
    }

    // Other characters
    result += escapeHtml(code[i]);
    i++;
  }

  return result;
}

function escapeHtml(str) {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}
