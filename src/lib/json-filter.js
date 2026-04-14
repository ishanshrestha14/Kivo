export function parseQuery(input) {
  if (!input) return null;
  const str = input.trim();
  
  const match = str.match(/^([\w.-]+)\s*(==|!=|>=|<=|>|<|=)\s*(.+)$/);
  if (match) {
    let key = match[1];
    let op = match[2];
    let valStr = match[3].trim();
    
    if (op === "=") op = "==";
    
    let val = valStr;
    if (valStr === "true") val = true;
    else if (valStr === "false") val = false;
    else if (valStr === "null") val = null;
    else if (!isNaN(Number(valStr)) && valStr !== "") val = Number(valStr);
    else if (valStr.startsWith('"') && valStr.endsWith('"')) val = valStr.slice(1, -1);
    else if (valStr.startsWith("'") && valStr.endsWith("'")) val = valStr.slice(1, -1);
    
    return { type: "condition", key, op, val };
  }
  
  return { type: "text", text: str.toLowerCase() };
}

export function evaluateCondition(nodeVal, op, queryVal) {
  if (op === "==") return nodeVal == queryVal;
  if (op === "!=") return nodeVal != queryVal;

  if (typeof nodeVal === 'number' && typeof queryVal === 'number') {
    if (op === ">") return nodeVal > queryVal;
    if (op === ">=") return nodeVal >= queryVal;
    if (op === "<") return nodeVal < queryVal;
    if (op === "<=") return nodeVal <= queryVal;
  }
  
  if (typeof nodeVal === 'string' && typeof queryVal === 'string') {
    if (op === ">") return nodeVal > queryVal;
    if (op === ">=") return nodeVal >= queryVal;
    if (op === "<") return nodeVal < queryVal;
    if (op === "<=") return nodeVal <= queryVal;
  }
  
  return false;
}

export function filterJson(data, inputStr) {
  const query = parseQuery(inputStr);
  if (!query) return data;

  function filterNode(node, nodeKey = null) {
    if (query.type === "text") {
      const q = query.text;
      if (nodeKey && String(nodeKey).toLowerCase().includes(q)) return node;
    } else {
      if (node !== null && typeof node === "object" && !Array.isArray(node)) {
        const lowerQueryKey = query.key.toLowerCase();
        let matchedKey = undefined;
        for (const k in node) {
          if (Object.prototype.hasOwnProperty.call(node, k)) {
            if (k.toLowerCase() === lowerQueryKey) {
              matchedKey = k;
              break;
            }
          }
        }
        
        if (matchedKey !== undefined) {
          if (evaluateCondition(node[matchedKey], query.op, query.val)) {
            return node;
          }
        }
      }
    }

    if (node === null) {
      if (query.type === "text" && "null".includes(query.text)) return node;
      return undefined;
    }

    if (typeof node === "object") {
      const isArray = Array.isArray(node);
      const res = isArray ? [] : {};
      let hasMatch = false;

      if (isArray) {
        for (let i = 0; i < node.length; i++) {
          const childNode = filterNode(node[i], null);
          if (childNode !== undefined) {
            hasMatch = true;
            res.push(childNode);
          }
        }
      } else {
        for (const k in node) {
          if (Object.prototype.hasOwnProperty.call(node, k)) {
            const childNode = filterNode(node[k], k);
            if (childNode !== undefined) {
              hasMatch = true;
              res[k] = childNode;
            }
          }
        }
      }
      return hasMatch ? res : undefined;
    }

    if (query.type === "text") {
      if (String(node).toLowerCase().includes(query.text)) return node;
    }
    
    return undefined;
  }

  const result = filterNode(data);
  return result === undefined ? (Array.isArray(data) ? [] : {}) : result;
}
