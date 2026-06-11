/**
 * FlowForge WebBridge — background.js (WebSocket 版)
 * 
 * 通过 WebSocket 与 FlowForge 后端通信，无需 Native Messaging。
 * 扩展主动连接到 ws://localhost:19529/ws/browser
 * 
 * 优势：零配置，不需要安装 native messaging host
 */

// ═══════════════════════════════════════════════
// 配置（端口从 chrome.storage 读取，可配置）
// ═══════════════════════════════════════════════

const DEFAULT_PORT = 19529;  // FlowForge default port
let WS_PORT = DEFAULT_PORT;
let WS_URL = `ws://127.0.0.1:${WS_PORT}/ws/browser`;
const RECONNECT_DELAY = 2000;
const COMMAND_TIMEOUT = 30000;
const PAPER_SIZES = { letter: [8.5, 11], legal: [8.5, 14], a4: [8.27, 11.69], a3: [11.69, 16.54], tabloid: [11, 17] };

// 从 storage 加载端口配置
async function loadPortConfig() {
    try {
        const result = await chrome.storage.sync.get(['wfPort']);
        if (result.wfPort) {
            WS_PORT = result.wfPort;
            WS_URL = `ws://127.0.0.1:${WS_PORT}/ws/browser`;
            console.log(`[WebBridge] 使用配置端口: ${WS_PORT}`);
        }
    } catch (e) {
        console.log(`[WebBridge] 使用默认端口: ${DEFAULT_PORT}`);
    }
}

// 监听端口配置变化
chrome.storage.onChanged.addListener((changes, area) => {
    if (area === 'sync' && changes.wfPort) {
        WS_PORT = changes.wfPort.newValue || DEFAULT_PORT;
        WS_URL = `ws://127.0.0.1:${WS_PORT}/ws/browser`;
        console.log(`[WebBridge] 端口已更新: ${WS_PORT}，重连中...`);
        if (ws) ws.close();
        connectWebSocket();
    }
});

// ═══════════════════════════════════════════════
// 状态管理
// ═══════════════════════════════════════════════

const attachedTabs = new Set();
let activeTabId = null;
let lastFocusedTabId = null;
let ws = null;
let wsConnected = false;

// Dialog 处理状态
let _dialogAction = 'accept';
let _dialogPromptText = '';

// Network 捕获
const networkCaptures = new Map(); // tabId -> Map<requestId, requestInfo>
const activeCaptures = new Set();  // 正在捕获的 tabId
let networkListenerAdded = false;

// Tab 分组
const sessionGroups = new Map();   // session -> groupId
const sessionColors = new Map();   // session -> color
// 站点预定义颜色（超越 Kimi：中文站点支持）
const SITE_COLORS = {
  github: 'blue', 'twitter.com': 'blue', 'x.com': 'blue',
  'xiaohongshu.com': 'red', 'xhslink.com': 'red',
  zhihu: 'blue', bilibili: 'cyan', weibo: 'orange',
  'google.com': 'green', baidu: 'blue',
  jira: 'blue', linear: 'purple', notion: 'grey',
};
const SESSION_COLORS = ['blue', 'red', 'yellow', 'green', 'cyan', 'orange', 'pink', 'purple'];
let sessionColorIdx = 0;

// 操作追踪
let _traceEnabled = false;
let _traceLog = [];

// @e ref 系统
const refMap = new Map();
let refCounter = 1;

const REF_ROLES = new Set([
  'button', 'link', 'textbox', 'checkbox', 'radio',
  'combobox', 'listbox', 'menuitem', 'menuitemcheckbox',
  'menuitemradio', 'option', 'searchbox', 'slider',
  'spinbutton', 'switch', 'tab', 'treeitem',
]);

// ═══════════════════════════════════════════════
// WebSocket 通信
// ═══════════════════════════════════════════════

function connectWebSocket() {
  if (ws && (ws.readyState === WebSocket.CONNECTING || ws.readyState === WebSocket.OPEN)) {
    return;
  }
  
  try {
    ws = new WebSocket(WS_URL);
    
    ws.onopen = () => {
      wsConnected = true;
      console.log('[WebBridge] Connected to FlowForge');
      // 注册自己
      ws.send(JSON.stringify({
        type: 'register',
        client: 'webbridge',
        version: '1.5.0',
        capabilities: Object.keys(tools),
      }));
    };
    
    ws.onmessage = async (event) => {
      try {
        const msg = JSON.parse(event.data);
        await handleCommand(msg);
      } catch (e) {
        console.error('[WebBridge] Failed to handle message:', e);
      }
    };
    
    ws.onclose = () => {
      wsConnected = false;
      console.log('[WebBridge] Disconnected, reconnecting...');
      setTimeout(connectWebSocket, RECONNECT_DELAY);
      // alarms 兜底（service worker 被挂起后 setTimeout 不执行）
      chrome.alarms?.create('webbridge-reconnect', { delayInMinutes: 1 });
    };
    
    ws.onerror = (e) => {
      console.error('[WebBridge] WebSocket error:', e);
    };
  } catch (e) {
    console.error('[WebBridge] Failed to connect:', e);
    setTimeout(connectWebSocket, RECONNECT_DELAY);
  }
}

async function handleCommand(msg) {
  const { id, action, params = {} } = msg;
  
  if (!id || !action) return;
  
  try {
    if (!tools[action]) {
      throw new Error(`Unknown action: ${action}`);
    }
    
    // _tabId 全局路由：任何命令可带 _tabId 指定目标 tab（对齐 Kimi）
    const tabId = params._tabId;
    if (tabId != null && action !== 'close_tab' && action !== 'list_tabs' && action !== 'close_session') {
      await ensureAttached(tabId);
      lastFocusedTabId = tabId;
      delete params._tabId;
    }

    const result = await tools[action](params);
    // Trace logging
    if (_traceEnabled) {
      _traceLog.push({ timestamp: Date.now(), action, params: JSON.stringify(params).slice(0, 200), success: true });
      if (_traceLog.length > 500) _traceLog.shift();
    }
    ws?.send(JSON.stringify({ id, success: true, data: result }));
  } catch (e) {
    if (_traceEnabled) {
      _traceLog.push({ timestamp: Date.now(), action, params: JSON.stringify(params).slice(0, 200), success: false, error: e.message });
      if (_traceLog.length > 500) _traceLog.shift();
    }
    ws?.send(JSON.stringify({ id, success: false, error: e.message }));
  }
}

// ═══════════════════════════════════════════════
// CDP 会话管理
// ═══════════════════════════════════════════════

async function ensureAttached(tabId) {
  if (attachedTabs.has(tabId)) {
    activeTabId = tabId;
    return;
  }
  // 尝试获取调试版本来判断是否已 attach（避免闪弹窗）
  try {
    const targets = await chrome.debugger.getTargets?.() || [];
    const alreadyAttached = targets.some(t => t.tabId === tabId && t.attached);
    if (alreadyAttached) {
      attachedTabs.add(tabId);
      activeTabId = tabId;
      return;
    }
  } catch {}
  try {
    await chrome.debugger.detach({ tabId });
  } catch {}
  await chrome.debugger.attach({ tabId }, '1.3');
  attachedTabs.add(tabId);
  activeTabId = tabId;
}

async function cdpCommand(method, params = {}) {
  if (activeTabId === null) {
    throw new Error('No tab attached. Call attach(tabId) first.');
  }
  return await chrome.debugger.sendCommand({ tabId: activeTabId }, method, params);
}

async function resolveTab() {
  if (activeTabId !== null) {
    try {
      const tab = await chrome.tabs.get(activeTabId);
      if (tab) return tab;
    } catch {
      attachedTabs.delete(activeTabId);
      activeTabId = null;
    }
  }
  if (lastFocusedTabId !== null) {
    try {
      const tab = await chrome.tabs.get(lastFocusedTabId);
      if (tab) return tab;
    } catch {
      lastFocusedTabId = null;
    }
  }
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id) throw new Error('No active tab found');
  lastFocusedTabId = tab.id;
  return tab;
}

// ═══════════════════════════════════════════════
// @e ref 系统
// ═══════════════════════════════════════════════

function resetRefs() {
  refMap.clear();
  refCounter = 1;
}

function makeRef(backendDOMNodeId, role, name) {
  const refId = `e${refCounter++}`;
  refMap.set(refId, { backendDOMNodeId, role, name });
  return refId;
}

function resolveRef(ref) {
  const id = ref.startsWith('@') ? ref.slice(1) : ref;
  return refMap.get(id);
}

function isRef(str) {
  return /^@?e\d+$/.test(str);
}

// ═══════════════════════════════════════════════
// 工具实现（与 Kimi WebBridge 完全兼容）
// ═══════════════════════════════════════════════

const tools = {
  // ─── 浏览器管理 ───
  
  async navigate(params) {
    let url = params.url;
    if (!url) throw new Error('navigate: url is required');
    
    const newTab = params.newTab;
    const session = params._session;
    const groupTitle = params.group_title;
    const tab = await resolveTab();
    
    if (newTab) {
      const created = await chrome.tabs.create({ url, active: true });
      lastFocusedTabId = created.id;
      if (session) await assignToSession(created.id, session, groupTitle);
      await ensureAttached(created.id);
      await waitForLoad(created.id);
      return { success: true, url, tabId: created.id };
    }
    
    // chrome:// / edge:// 协议不能 CDP 导航，必须开新 tab
    if (tab.url?.startsWith('chrome://') || tab.url?.startsWith('edge://')) {
      const created = await chrome.tabs.create({ url, active: true });
      lastFocusedTabId = created.id;
      if (session) await assignToSession(created.id, session, groupTitle);
      await ensureAttached(created.id);
      await waitForLoad(created.id);
      return { success: true, url, tabId: created.id };
    }

    await ensureAttached(tab.id);
    // 同 URL 则刷新而非重新导航（对齐 Kimi）
    const normalizeUrl = (u) => u?.replace(/\/+$/, '').replace(/\?$/, '') || '';
    const isSameUrl = normalizeUrl(tab.url) === normalizeUrl(url);
    let frameId;
    if (isSameUrl) {
      await cdpCommand('Page.reload', { ignoreCache: true });
    } else {
      const navResult = await cdpCommand('Page.navigate', { url });
      frameId = navResult.frameId;
    }
    await waitForLoad(tab.id);
    // 重新查询获取导航后的实际 URL
    const updated = await chrome.tabs.get(tab.id);
    const result = { success: true, url: updated.url || url, tabId: tab.id };
    if (frameId) result.frameId = frameId;
    return result;
  },

  async find_tab(params) {
    const { url, title, index, _session, active: wantActive } = params;
    const tabs = await chrome.tabs.query({});
    
    let target;
    if (index !== undefined) {
      target = tabs[index];
    } else if (url) {
      // 对齐 Kimi: URL 自动转 glob pattern 用 chrome.tabs.query 高效查找
      const pattern = urlToGlobPattern(url);
      const matched = await chrome.tabs.query({ url: pattern });
      if (wantActive) {
        // 优先找活跃 tab
        const activeMatched = matched.find(t => t.active);
        target = activeMatched || matched[0];
      }
      target = target || matched[0];
      // fallback: 遍历匹配
      if (!target) target = tabs.find(t => t.url?.includes(url));
    } else if (title) {
      target = tabs.find(t => t.title?.includes(title));
    }
    
    if (!target) throw new Error(`Tab not found: ${JSON.stringify(params)}`);
    if (_session) await assignToSession(target.id, _session);
    lastFocusedTabId = target.id;
    await ensureAttached(target.id);
    return { success: true, tabId: target.id, url: target.url, title: target.title };
  },

  async list_tabs() {
    const tabs = await chrome.tabs.query({});
    const result = [];
    for (const t of tabs) {
      let groupTitle;
      if (t.groupId != null && t.groupId !== chrome.tabGroups?.TAB_GROUP_ID_NONE) {
        try {
          const group = await chrome.tabGroups.get(t.groupId);
          groupTitle = group.title;
        } catch {}
      }
      result.push({
        tabId: t.id,
        url: t.url,
        title: t.title,
        active: t.active,
        groupTitle,
      });
    }
    return result;
  },

  async close_tab(params) {
    const tabId = params.tabId || activeTabId;
    if (!tabId) throw new Error('No tab to close');
    await chrome.tabs.remove(tabId);
    attachedTabs.delete(tabId);
    if (activeTabId === tabId) activeTabId = null;
    return { success: true, closed: tabId };
  },

  async close_session() {
    for (const tabId of attachedTabs) {
      try { await chrome.debugger.detach({ tabId }); } catch {}
    }
    attachedTabs.clear();
    activeTabId = null;
    // 清理 network 捕获
    networkCaptures.clear();
    activeCaptures.clear();
    return { success: true, message: 'Session closed' };
  },

  // ─── 页面操作 ───

  async snapshot() {
    await ensureAttached((await resolveTab()).id);
    resetRefs();
    
    const axResult = await cdpCommand('Accessibility.getFullAXTree');
    const nodes = axResult.nodes || [];
    const tree = buildAxTree(nodes);
    
    const tab = await resolveTab();
    return {
      url: tab.url,
      title: tab.title,
      tree,
      refs: Object.fromEntries(
        [...refMap.entries()].map(([k, v]) => [k, { role: v.role, name: v.name }])
      ),
    };
  },

  async click(params) {
    const { selector } = params;
    if (!selector) throw new Error('click: selector is required');
    await ensureAttached((await resolveTab()).id);
    
    if (isRef(selector)) {
      return await clickByRef(selector);
    }
    return await clickBySelector(selector);
  },

  async fill(params) {
    const { selector, value } = params;
    if (!selector) throw new Error('fill: selector is required');
    if (value == null) throw new Error('fill: value is required');
    await ensureAttached((await resolveTab()).id);
    
    if (isRef(selector)) {
      return await fillByRef(selector, value);
    }
    return await fillBySelector(selector, value);
  },

  async evaluate(params) {
    const { code, expression } = params;
    const script = code || expression;
    if (!script) throw new Error('evaluate: code/expression is required');
    await ensureAttached((await resolveTab()).id);
    
    const result = await cdpCommand('Runtime.evaluate', {
      expression: script,
      returnByValue: true,
      awaitPromise: true,
    });
    
    if (result.exceptionDetails) {
      const desc = result.exceptionDetails.exception?.description || result.exceptionDetails.text;
      throw new Error(`evaluate: ${desc}`);
    }
    return { type: typeof result.result.value, value: result.result.value };
  },

  async screenshot(params) {
    await ensureAttached((await resolveTab()).id);
    const format = params.format || 'png';
    const quality = params.quality;
    const selector = params.selector;
    
    const opts = { format };
    if (quality && format === 'jpeg') opts.quality = quality;
    
    // 元素级截图（对齐 Kimi）
    if (selector) {
      const objectId = isRef(selector)
        ? await objectIdFromRef(selector)
        : await objectIdFromSelector(selector);
      await cdpCommand('Runtime.callFunctionOn', {
        objectId,
        functionDeclaration: "function() { this.scrollIntoView({block:'center',inline:'center'}); }",
      });
      const box = await cdpCommand('DOM.getBoxModel', { objectId });
      const border = box.model?.border;
      if (!border || border.length < 8) throw new Error('Element has no layout box');
      const xs = [border[0], border[2], border[4], border[6]];
      const ys = [border[1], border[3], border[5], border[7]];
      const x = Math.min(...xs), y = Math.min(...ys);
      const w = Math.max(...xs) - x, h = Math.max(...ys) - y;
      if (w <= 0 || h <= 0) throw new Error(`Element has zero-size box (${w}x${h})`);
      opts.clip = { x, y, width: w, height: h, scale: 1 };
    }

    const result = await cdpCommand('Page.captureScreenshot', opts);
    return { data: result.data, format };
  },

  async save_as_pdf(params) {
    await ensureAttached((await resolveTab()).id);
    const [paperW, paperH] = PAPER_SIZES[(params.paper_format || 'letter').toLowerCase()] || PAPER_SIZES.letter;
    const scale = typeof params.scale === 'number' ? Math.max(0.1, Math.min(2, params.scale)) : 1;
    const result = await cdpCommand('Page.printToPDF', {
      printBackground: true,
      preferCSSPageSize: true,
      landscape: !!params.landscape,
      scale,
      paperWidth: paperW,
      paperHeight: paperH,
    });
    return { data: result.data };
  },

  // ─── 键盘/鼠标 ───

  async mouse_click(params) {
    const { selector, x, y } = params;
    await ensureAttached((await resolveTab()).id);
    
    let cx, cy;
    if (x !== undefined && y !== undefined) {
      cx = x; cy = y;
    } else if (selector) {
      const objectId = isRef(selector)
        ? await objectIdFromRef(selector)
        : await objectIdFromSelector(selector);
      
      await cdpCommand('Runtime.callFunctionOn', {
        objectId,
        functionDeclaration: "function() { this.scrollIntoView({block:'center',inline:'center'}); }",
      });
      
      const box = await cdpCommand('DOM.getBoxModel', { objectId });
      const c = box.model?.content;
      if (!c || c.length < 8) throw new Error('mouse_click: element has no layout box (display:none/detached/zero-size). Use click for DOM-level fallback.');
      cx = (c[0] + c[2] + c[4] + c[6]) / 4;
      cy = (c[1] + c[3] + c[5] + c[7]) / 4;
    } else {
      throw new Error('mouse_click: need selector or x,y coordinates');
    }
    
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseMoved', x: cx, y: cy, button: 'none', buttons: 0 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mousePressed', x: cx, y: cy, button: 'left', buttons: 1, clickCount: 1 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseReleased', x: cx, y: cy, button: 'left', buttons: 0, clickCount: 1 });
    
    return { success: true, x: Math.round(cx), y: Math.round(cy) };
  },

  async key_type(params) {
    const { text, delay } = params;
    if (!text) throw new Error('key_type: text is required');
    await ensureAttached((await resolveTab()).id);
    
    if (delay) {
      // 逐字符模式（需要延迟控制时）
      for (const char of text) {
        await cdpCommand('Input.dispatchKeyEvent', {
          type: 'keyDown', text: char, key: char,
        });
        await cdpCommand('Input.dispatchKeyEvent', {
          type: 'keyUp', key: char,
        });
        await sleep(delay);
      }
    } else {
      // 高效模式：一次调用插入全部文本（对齐 Kimi）
      await cdpCommand('Input.insertText', { text });
    }
    return { success: true, typed: text.length };
  },

  async send_keys(params) {
    const { key, modifiers } = params;
    if (!key) throw new Error('send_keys: key is required');
    await ensureAttached((await resolveTab()).id);
    
    const repeat = Math.min(Math.max(params.repeat || 1, 1), 100);
    const spec = resolveKeySpec(key);
    const modBits = resolveModifiers(modifiers || []);
    
    let dispatched = 0;
    for (let r = 0; r < repeat; r++) {
      // 按下修饰键
      let activeBits = 0;
      for (const mod of spec.modKeys || []) {
        activeBits |= mod.bit;
        await cdpCommand('Input.dispatchKeyEvent', {
          type: 'keyDown', modifiers: activeBits,
          key: mod.key, code: mod.code,
          windowsVirtualKeyCode: mod.vkc,
        });
      }
      // 按下主键
      const mainMods = modBits | (spec.modKeys || []).reduce((a, m) => a | m.bit, 0);
      const keyDownParams = {
        type: 'keyDown', modifiers: mainMods,
        key: spec.key, code: spec.code,
        windowsVirtualKeyCode: spec.vkc,
      };
      if (spec.text !== undefined && mainMods === 0) keyDownParams.text = spec.text;
      await cdpCommand('Input.dispatchKeyEvent', keyDownParams);
      await cdpCommand('Input.dispatchKeyEvent', {
        type: 'keyUp', modifiers: mainMods,
        key: spec.key, code: spec.code,
        windowsVirtualKeyCode: spec.vkc,
      });
      // 释放修饰键（逆序）
      for (const mod of (spec.modKeys || []).slice().reverse()) {
        activeBits &= ~mod.bit;
        await cdpCommand('Input.dispatchKeyEvent', {
          type: 'keyUp', modifiers: activeBits,
          key: mod.key, code: mod.code,
          windowsVirtualKeyCode: mod.vkc,
        });
      }
      dispatched++;
    }
    return { success: true, key, dispatched };
  },

  // ─── 超越 Kimi：新增命令 ───

  async scroll(params) {
    const { selector, x = 0, y = 500, direction } = params;
    await ensureAttached((await resolveTab()).id);
    let scrollX = x, scrollY = y;
    if (direction === 'up') scrollY = -Math.abs(y);
    else if (direction === 'down') scrollY = Math.abs(y);
    else if (direction === 'left') scrollX = -Math.abs(x);
    else if (direction === 'right') scrollX = Math.abs(x);
    if (selector) {
      const objectId = isRef(selector) ? await objectIdFromRef(selector) : await objectIdFromSelector(selector);
      await cdpCommand('Runtime.callFunctionOn', {
        objectId,
        functionDeclaration: `function() { this.scrollBy(${scrollX}, ${scrollY}); }`,
      });
    } else {
      await cdpCommand('Runtime.evaluate', {
        expression: `window.scrollBy(${scrollX}, ${scrollY})`,
      });
    }
    return { success: true, scrolled: { x: scrollX, y: scrollY } };
  },

  async hover(params) {
    const { selector } = params;
    if (!selector) throw new Error('hover: selector is required');
    await ensureAttached((await resolveTab()).id);
    const objectId = isRef(selector) ? await objectIdFromRef(selector) : await objectIdFromSelector(selector);
    await cdpCommand('Runtime.callFunctionOn', {
      objectId,
      functionDeclaration: "function() { this.scrollIntoView({block:'center',inline:'center'}); }",
    });
    const box = await cdpCommand('DOM.getBoxModel', { objectId });
    const c = box.model?.content;
    if (!c || c.length < 8) throw new Error('hover: element has no layout box');
    const cx = (c[0] + c[2] + c[4] + c[6]) / 4;
    const cy = (c[1] + c[3] + c[5] + c[7]) / 4;
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseMoved', x: cx, y: cy, button: 'none', buttons: 0 });
    return { success: true, x: Math.round(cx), y: Math.round(cy) };
  },

  async go_back() {
    await ensureAttached((await resolveTab()).id);
    await cdpCommand('Runtime.evaluate', { expression: 'history.back()' });
    await sleep(500);
    const tab = await chrome.tabs.get(activeTabId);
    return { success: true, url: tab.url };
  },

  async go_forward() {
    await ensureAttached((await resolveTab()).id);
    await cdpCommand('Runtime.evaluate', { expression: 'history.forward()' });
    await sleep(500);
    const tab = await chrome.tabs.get(activeTabId);
    return { success: true, url: tab.url };
  },

  async wait_for(params) {
    const { selector, text, timeout = 10000 } = params;
    if (!selector && !text) throw new Error('wait_for: selector or text is required');
    await ensureAttached((await resolveTab()).id);
    const start = Date.now();
    while (Date.now() - start < timeout) {
      if (selector) {
        const r = await cdpCommand('Runtime.evaluate', {
          expression: `!!document.querySelector(${JSON.stringify(selector)})`,
          returnByValue: true,
        });
        if (r.result?.value === true) return { success: true, selector, elapsed: Date.now() - start };
      }
      if (text) {
        const r = await cdpCommand('Runtime.evaluate', {
          expression: `document.body?.innerText?.includes(${JSON.stringify(text)})`,
          returnByValue: true,
        });
        if (r.result?.value === true) return { success: true, text, elapsed: Date.now() - start };
      }
      await sleep(300);
    }
    throw new Error(`wait_for: timeout (${timeout}ms) waiting for ${selector || JSON.stringify(text)}`);
  },

  // ─── 网络 ───

  async network(params) {
    await ensureAttached((await resolveTab()).id);
    const cmd = params.cmd || params.action || 'start';

    switch (cmd) {
      case 'start': {
        ensureNetworkListener();
        networkCaptures.set(activeTabId, new Map());
        activeCaptures.add(activeTabId);
        await cdpCommand('Network.enable');
        return { success: true, message: 'network capture started' };
      }
      case 'stop': {
        activeCaptures.delete(activeTabId);
        try { await cdpCommand('Network.disable'); } catch {}
        return { success: true, message: 'network capture stopped' };
      }
      case 'list': {
        const captures = networkCaptures.get(activeTabId);
        let requests = captures ? [...captures.values()] : [];
        if (params.filter) {
          requests = requests.filter(r => r.url.includes(params.filter));
        }
        return {
          count: requests.length,
          requests: requests.map(r => ({
            requestId: r.requestId,
            url: r.url,
            method: r.method,
            status: r.status,
            mimeType: r.mimeType,
            completed: r.completed ?? false,
          })),
        };
      }
      case 'detail': {
        const reqId = params.requestId;
        if (!reqId) throw new Error('network detail: requestId is required');
        const captures = networkCaptures.get(activeTabId);
        const req = captures?.get(reqId);
        if (!req) throw new Error(`network: request "${reqId}" not found`);
        const resp = await cdpCommand('Network.getResponseBody', { requestId: reqId });
        let body = resp.body;
        if (!resp.base64Encoded) {
          try { body = JSON.parse(resp.body); } catch {}
        }
        return {
          requestId: req.requestId,
          url: req.url,
          method: req.method,
          status: req.status,
          mimeType: req.mimeType,
          base64Encoded: resp.base64Encoded,
          body,
        };
      }
      default:
        throw new Error(`network: unknown cmd "${cmd}"`);
    }
  },

  async upload(params) {
    const { selector, filePaths } = params;
    if (!selector) throw new Error('upload: selector is required');
    if (!filePaths?.length) throw new Error('upload: filePaths is required');
    await ensureAttached((await resolveTab()).id);
    
    const doc = await cdpCommand('DOM.getDocument');
    const { nodeId } = await cdpCommand('DOM.querySelector', {
      nodeId: doc.root.nodeId,
      selector,
    });
    if (!nodeId) throw new Error(`upload: element not found: ${selector}`);
    await cdpCommand('DOM.setFileInputFiles', { files: filePaths, nodeId });
    return { success: true, files: filePaths.length };
  },

  async cdp(params) {
    const { method, params: cdpParams } = params;
    if (!method) throw new Error('cdp: method is required');
    await ensureAttached((await resolveTab()).id);
    return await cdpCommand(method, cdpParams || {});
  },

  async download(params) {
    const { selector, url, saveAs } = params;
    await ensureAttached((await resolveTab()).id);
    
    // 设置下载行为
    const downloadPath = saveAs || 'Downloads';
    await cdpCommand('Page.setDownloadBehavior', {
      behavior: 'allow',
      downloadPath: downloadPath,
    });
    
    if (url) {
      // 直接通过 URL 下载
      await cdpCommand('Page.navigate', { url });
      await waitForLoad(activeTabId);
      return { success: true, method: 'navigate', url };
    }
    
    if (selector) {
      // 通过点击元素触发下载
      if (isRef(selector)) {
        await clickByRef(selector);
      } else {
        await clickBySelector(selector);
      }
      return { success: true, method: 'click', selector };
    }
    
    throw new Error('download: need url or selector');
  },

  // ─── Phase 1: 数据提取 + 元信息 ───

  async extract_text(params) {
    const { selector, ref } = params;
    if (!selector && !ref) throw new Error('extract_text: selector or ref is required');
    await ensureAttached((await resolveTab()).id);
    const objectId = ref ? await objectIdFromRef(ref) : await objectIdFromSelector(selector || ref);
    const result = await cdpCommand('Runtime.callFunctionOn', {
      objectId,
      functionDeclaration: "function() { return this.innerText || this.textContent || ''; }",
      returnByValue: true,
    });
    return { selector: selector || ref, text: result.result.value };
  },

  async extract_html(params) {
    const { selector, ref } = params;
    if (!selector && !ref) throw new Error('extract_html: selector or ref is required');
    await ensureAttached((await resolveTab()).id);
    const objectId = ref ? await objectIdFromRef(ref) : await objectIdFromSelector(selector || ref);
    const result = await cdpCommand('Runtime.callFunctionOn', {
      objectId,
      functionDeclaration: "function() { return this.outerHTML; }",
      returnByValue: true,
    });
    return result.result.value;
  },

  async extract_attribute(params) {
    const { selector, ref, attribute } = params;
    if (!selector && !ref) throw new Error('extract_attribute: selector or ref is required');
    if (!attribute) throw new Error('extract_attribute: attribute is required');
    await ensureAttached((await resolveTab()).id);
    const objectId = ref ? await objectIdFromRef(ref) : await objectIdFromSelector(selector || ref);
    const result = await cdpCommand('Runtime.callFunctionOn', {
      objectId,
      functionDeclaration: `function() { return this.getAttribute(${JSON.stringify(attribute)}); }`,
      returnByValue: true,
    });
    return { selector: selector || ref, attribute, value: result.result.value };
  },

  async extract_links(params) {
    const { selector = 'a[href]' } = params;
    await ensureAttached((await resolveTab()).id);
    const result = await cdpCommand('Runtime.evaluate', {
      expression: `JSON.stringify([...document.querySelectorAll(${JSON.stringify(selector)}).map(a => ({text: a.innerText.trim(), href: a.href}))])`,
      returnByValue: true,
    });
    const items = JSON.parse(result.result.value);
    return { count: items.length, items };
  },

  async extract_table(params) {
    const { selector = 'table', ref } = params;
    await ensureAttached((await resolveTab()).id);
    const expr = ref
      ? `(() => {
          const el = document.querySelector('[data-ref="${ref}"]') || document.activeElement;
          if (!el || el.tagName !== 'TABLE') return JSON.stringify({error: 'table not found'});
          const headers = [...el.querySelectorAll('th')].map(th => th.innerText.trim());
          const rows = [...el.querySelectorAll('tbody tr')].map(tr =>
            [...tr.querySelectorAll('td')].map(td => td.innerText.trim())
          );
          return JSON.stringify({headers, rows});
        })()`
      : `(() => {
          const el = document.querySelector(${JSON.stringify(selector)});
          if (!el) return JSON.stringify({error: 'table not found'});
          const headers = [...el.querySelectorAll('th')].map(th => th.innerText.trim());
          const rows = [...el.querySelectorAll('tbody tr')].map(tr =>
            [...tr.querySelectorAll('td')].map(td => td.innerText.trim())
          );
          return JSON.stringify({headers, rows});
        })()`;
    const result = await cdpCommand('Runtime.evaluate', { expression: expr, returnByValue: true });
    return JSON.parse(result.result.value);
  },

  async get_title() {
    await ensureAttached((await resolveTab()).id);
    const result = await cdpCommand('Runtime.evaluate', {
      expression: 'document.title',
      returnByValue: true,
    });
    return { title: result.result.value };
  },

  async current_url() {
    await ensureAttached((await resolveTab()).id);
    const result = await cdpCommand('Runtime.evaluate', {
      expression: 'location.href',
      returnByValue: true,
    });
    return { url: result.result.value };
  },

  async reload(params) {
    await ensureAttached((await resolveTab()).id);
    await cdpCommand('Page.reload', { ignoreCache: params?.ignoreCache || false });
    if (params?.waitUntil !== false) {
      await waitForLoad(activeTabId);
    }
    const tab = await chrome.tabs.get(activeTabId);
    return { success: true, url: tab.url };
  },

  // ─── Phase 2: 表单 + 鼠标交互 ───

  async select(params) {
    const { selector, value } = params;
    if (!selector) throw new Error('select: selector is required');
    if (value == null) throw new Error('select: value is required');
    await ensureAttached((await resolveTab()).id);
    const result = await cdpCommand('Runtime.evaluate', {
      expression: `(() => {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) return { error: 'element not found: ' + ${JSON.stringify(selector)} };
        if (el.tagName !== 'SELECT') return { error: 'not a select element: ' + el.tagName };
        el.value = ${JSON.stringify(value)};
        el.dispatchEvent(new Event('change', { bubbles: true }));
        return { success: true, selector: ${JSON.stringify(selector)}, value: ${JSON.stringify(value)} };
      })()`,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error(`select: ${result.exceptionDetails.text}`);
    const val = result.result.value;
    if (val?.error) throw new Error(val.error);
    return val;
  },

  async check(params) {
    const { selector, checked = true } = params;
    if (!selector) throw new Error('check: selector is required');
    await ensureAttached((await resolveTab()).id);
    const result = await cdpCommand('Runtime.evaluate', {
      expression: `(() => {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) return { error: 'element not found: ' + ${JSON.stringify(selector)} };
        const target = ${JSON.stringify(checked)};
        if (el.checked !== target) el.click();
        return { success: true, selector: ${JSON.stringify(selector)}, checked: el.checked };
      })()`,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error(`check: ${result.exceptionDetails.text}`);
    const val = result.result.value;
    if (val?.error) throw new Error(val.error);
    return val;
  },

  async double_click(params) {
    const { selector } = params;
    if (!selector) throw new Error('double_click: selector is required');
    await ensureAttached((await resolveTab()).id);
    const objectId = isRef(selector) ? await objectIdFromRef(selector) : await objectIdFromSelector(selector);
    await cdpCommand('Runtime.callFunctionOn', {
      objectId,
      functionDeclaration: "function() { this.scrollIntoView({block:'center',inline:'center'}); }",
    });
    const box = await cdpCommand('DOM.getBoxModel', { objectId });
    const c = box.model?.content;
    if (!c || c.length < 8) throw new Error('double_click: element has no layout box');
    const cx = (c[0] + c[2] + c[4] + c[6]) / 4;
    const cy = (c[1] + c[3] + c[5] + c[7]) / 4;
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseMoved', x: cx, y: cy, button: 'none', buttons: 0 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mousePressed', x: cx, y: cy, button: 'left', buttons: 1, clickCount: 1 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseReleased', x: cx, y: cy, button: 'left', buttons: 0, clickCount: 1 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mousePressed', x: cx, y: cy, button: 'left', buttons: 1, clickCount: 2 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseReleased', x: cx, y: cy, button: 'left', buttons: 0, clickCount: 2 });
    return { success: true, selector, x: Math.round(cx), y: Math.round(cy) };
  },

  async context_menu(params) {
    const { selector } = params;
    if (!selector) throw new Error('context_menu: selector is required');
    await ensureAttached((await resolveTab()).id);
    const objectId = isRef(selector) ? await objectIdFromRef(selector) : await objectIdFromSelector(selector);
    await cdpCommand('Runtime.callFunctionOn', {
      objectId,
      functionDeclaration: "function() { this.scrollIntoView({block:'center',inline:'center'}); }",
    });
    const box = await cdpCommand('DOM.getBoxModel', { objectId });
    const c = box.model?.content;
    if (!c || c.length < 8) throw new Error('context_menu: element has no layout box');
    const cx = (c[0] + c[2] + c[4] + c[6]) / 4;
    const cy = (c[1] + c[3] + c[5] + c[7]) / 4;
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseMoved', x: cx, y: cy, button: 'none', buttons: 0 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mousePressed', x: cx, y: cy, button: 'right', buttons: 2, clickCount: 1 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseReleased', x: cx, y: cy, button: 'right', buttons: 0, clickCount: 1 });
    return { success: true, selector, x: Math.round(cx), y: Math.round(cy) };
  },

  async drag_to(params) {
    const { source, target, source_position, target_position } = params;
    if (!source) throw new Error('drag_to: source is required');
    if (!target) throw new Error('drag_to: target is required');
    await ensureAttached((await resolveTab()).id);

    async function getCenter(selector, pos) {
      if (pos?.x !== undefined && pos?.y !== undefined) return { x: pos.x, y: pos.y };
      const objectId = isRef(selector) ? await objectIdFromRef(selector) : await objectIdFromSelector(selector);
      await cdpCommand('Runtime.callFunctionOn', {
        objectId,
        functionDeclaration: "function() { this.scrollIntoView({block:'center',inline:'center'}); }",
      });
      const box = await cdpCommand('DOM.getBoxModel', { objectId });
      const c = box.model?.content;
      if (!c || c.length < 8) throw new Error(`drag_to: element "${selector}" has no layout box`);
      return {
        x: (c[0] + c[2] + c[4] + c[6]) / 4,
        y: (c[1] + c[3] + c[5] + c[7]) / 4,
      };
    }

    const from = await getCenter(source, source_position);
    const to = await getCenter(target, target_position);

    // Drag sequence: move to source → press → move to target → release
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseMoved', x: from.x, y: from.y, button: 'none', buttons: 0 });
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mousePressed', x: from.x, y: from.y, button: 'left', buttons: 1, clickCount: 1 });
    // Move in steps for smooth drag
    const steps = 5;
    for (let i = 1; i <= steps; i++) {
      const x = from.x + (to.x - from.x) * (i / steps);
      const y = from.y + (to.y - from.y) * (i / steps);
      await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y, button: 'left', buttons: 1 });
    }
    await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseReleased', x: to.x, y: to.y, button: 'left', buttons: 0, clickCount: 1 });
    return { success: true, from: { x: Math.round(from.x), y: Math.round(from.y) }, to: { x: Math.round(to.x), y: Math.round(to.y) } };
  },

  // ─── Phase 3: 标签管理 + 网络 + 对话框 ───

  async new_page(params) {
    const { url } = params || {};
    const tab = await chrome.tabs.create({ url: url || 'about:blank', active: true });
    lastFocusedTabId = tab.id;
    await ensureAttached(tab.id);
    if (url && url !== 'about:blank') await waitForLoad(tab.id);
    const updated = await chrome.tabs.get(tab.id);
    return { tabId: updated.id, url: updated.url, title: updated.title };
  },

  async switch_page(params) {
    const { index, tabId } = params || {};
    let targetId = tabId;
    if (targetId == null && index !== undefined) {
      const tabs = await chrome.tabs.query({});
      if (index >= tabs.length) throw new Error(`switch_page: index ${index} out of range (${tabs.length} tabs)`);
      targetId = tabs[index].id;
    }
    if (targetId == null) throw new Error('switch_page: need index or tabId');
    await chrome.tabs.update(targetId, { active: true });
    lastFocusedTabId = targetId;
    await ensureAttached(targetId);
    const tab = await chrome.tabs.get(targetId);
    return { tabId: tab.id, url: tab.url, title: tab.title };
  },

  async cookies(params) {
    const { action = 'get', cookies: setCookies } = params || {};
    await ensureAttached((await resolveTab()).id);
    await cdpCommand('Network.enable');

    if (action === 'get') {
      const result = await cdpCommand('Network.getCookies');
      return { cookies: result.cookies, count: result.cookies.length };
    }
    if (action === 'set') {
      if (!setCookies?.length) throw new Error('cookies set: cookies array required');
      for (const c of setCookies) {
        await cdpCommand('Network.setCookie', c);
      }
      return { success: true, set: setCookies.length };
    }
    if (action === 'clear') {
      await cdpCommand('Network.clearBrowserCookies');
      return { success: true, message: 'all cookies cleared' };
    }
    throw new Error(`cookies: unknown action "${action}"`);
  },

  async set_headers(params) {
    const { headers } = params || {};
    if (!headers || typeof headers !== 'object') throw new Error('set_headers: headers object required');
    await ensureAttached((await resolveTab()).id);
    await cdpCommand('Network.enable');
    await cdpCommand('Network.setExtraHTTPHeaders', { headers });
    return { success: true, headers };
  },

  async handle_dialog(params) {
    const { action = 'accept', prompt_text = '' } = params || {};
    // Store dialog handler preference — actual handling is event-based
    // This sets up the next dialog to be auto-handled
    _dialogAction = action;
    _dialogPromptText = prompt_text;
    // Enable dialog handling via CDP
    await ensureAttached((await resolveTab()).id);
    await cdpCommand('Page.enable');
    return { success: true, action, message: `Dialog handler set to "${action}"` };
  },

  // ─── Phase 4: Kimi 对齐 — observe→act→verify ───

  /**
   * press_key — 单键按下（对齐 Kimi）
   * params: { key: "Enter" | "Tab" | "Escape" | "Backspace" | ... }
   */
  async press_key(params) {
    const { key } = params;
    if (!key) throw new Error('press_key: key is required');
    return await tools.send_keys({ key, modifiers: params.modifiers });
  },

  /**
   * key_combo — 组合键（对齐 Kimi）
   * params: { keys: ["Control", "s"] } 或 { keys: ["ctrl", "shift", "p"] }
   */
  async key_combo(params) {
    const { keys } = params;
    if (!keys || !Array.isArray(keys) || keys.length < 2) {
      throw new Error('key_combo: keys array with ≥2 entries required, e.g. ["Control", "s"]');
    }
    const modifiers = [];
    let mainKey = null;
    const MOD_ALIASES = { ctrl: 'Control', control: 'Control', shift: 'Shift', alt: 'Alt', meta: 'Meta', cmd: 'Meta', win: 'Meta' };
    for (const k of keys) {
      const norm = k.charAt(0).toUpperCase() + k.slice(1).toLowerCase();
      const resolved = MOD_ALIASES[norm.toLowerCase()] || norm;
      if (['Control', 'Shift', 'Alt', 'Meta'].includes(resolved)) {
        modifiers.push(resolved);
      } else {
        mainKey = k; // preserve original case for the main key
      }
    }
    if (!mainKey) throw new Error('key_combo: no main key found (all are modifiers)');
    return await tools.send_keys({ key: mainKey, modifiers });
  },

  /**
   * submit_form — 智能表单提交（对齐 Kimi，5 策略 fallback）
   * params: { selector? } — 可选，指定 form 或其内部元素
   * 策略顺序：requestSubmit → 找 submit 按钮 → Enter → Ctrl+Enter → form.submit()
   */
  async submit_form(params) {
    const { selector } = params || {};
    await ensureAttached((await resolveTab()).id);

    const result = await cdpCommand('Runtime.evaluate', {
      expression: `(() => {
        function submitForm(el) {
          const form = el?.closest('form') || (el?.tagName === 'FORM' ? el : null);
          if (!form) return { error: 'no form found' };
          // Strategy 1: requestSubmit (fires submit event, respects validation)
          if (typeof form.requestSubmit === 'function') {
            try { form.requestSubmit(); return { success: true, strategy: 'requestSubmit' }; } catch {}
          }
          // Strategy 2: find submit button
          const btn = form.querySelector('button[type="submit"], input[type="submit"], button:not([type])');
          if (btn) { btn.click(); return { success: true, strategy: 'submitButton' }; }
          // Strategy 3: form.submit() (bypasses validation)
          form.submit();
          return { success: true, strategy: 'formSubmit' };
        }
        const el = ${selector ? `document.querySelector(${JSON.stringify(selector)})` : 'document.activeElement'};
        return submitForm(el);
      })()`,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error(`submit_form: ${result.exceptionDetails.text}`);
    const val = result.result.value;
    if (val?.error) throw new Error(`submit_form: ${val.error}`);
    return val;
  },

  /**
   * verify — 操作后验证（对齐 Kimi observe→act→verify）
   * params: { conditions: [{ type, ... }] }
   * condition types: url_changed, url_contains, text_visible, text_invisible, element_visible, element_invisible, element_count
   */
  async verify(params) {
    const { conditions } = params;
    if (!conditions || !Array.isArray(conditions)) throw new Error('verify: conditions array required');
    await ensureAttached((await resolveTab()).id);

    const results = [];
    for (const cond of conditions) {
      try {
        let expression;
        switch (cond.type) {
          case 'url_changed':
            expression = `location.href !== ${JSON.stringify(cond.from || '')}`;
            break;
          case 'url_contains':
            expression = `location.href.includes(${JSON.stringify(cond.value)})`;
            break;
          case 'text_visible':
            expression = `document.body?.innerText?.includes(${JSON.stringify(cond.value)}) || false`;
            break;
          case 'text_invisible':
            expression = `!(document.body?.innerText?.includes(${JSON.stringify(cond.value)}))`;
            break;
          case 'element_visible':
            expression = `(() => { const el = document.querySelector(${JSON.stringify(cond.selector)}); return el ? el.offsetParent !== null || el.getBoundingClientRect().height > 0 : false; })()`;
            break;
          case 'element_invisible':
            expression = `(() => { const el = document.querySelector(${JSON.stringify(cond.selector)}); return !el || el.offsetParent === null; })()`;
            break;
          case 'element_count':
            expression = `document.querySelectorAll(${JSON.stringify(cond.selector)}).length ${cond.operator || '>='} ${cond.count || 1}`;
            break;
          default:
            results.push({ type: cond.type, passed: false, error: 'unknown condition type' });
            continue;
        }
        const r = await cdpCommand('Runtime.evaluate', { expression, returnByValue: true });
        results.push({ type: cond.type, passed: !!r.result?.value });
      } catch (e) {
        results.push({ type: cond.type, passed: false, error: e.message });
      }
    }
    const allPassed = results.every(r => r.passed);
    return { passed: allPassed, conditions: results };
  },

  /**
   * wait_for_stable — 等待 DOM 稳定（对齐 Kimi MutationObserver）
   * params: { timeout?, stableMs? }
   * 当 DOM 在 stableMs 内无变化时返回
   */
  async wait_for_stable(params) {
    const { timeout = 10000, stableMs = 1500 } = params || {};
    await ensureAttached((await resolveTab()).id);

    // 注入 MutationObserver，等 DOM 停止变化
    const result = await cdpCommand('Runtime.evaluate', {
      expression: `new Promise((resolve, reject) => {
        let timer = null;
        const timeoutTimer = setTimeout(() => { observer.disconnect(); reject(new Error('timeout')); }, ${timeout});
        const observer = new MutationObserver(() => {
          clearTimeout(timer);
          timer = setTimeout(() => {
            observer.disconnect();
            clearTimeout(timeoutTimer);
            resolve({ success: true, stableMs: ${stableMs} });
          }, ${stableMs});
        });
        observer.observe(document.body, { childList: true, subtree: true, attributes: true, characterData: true });
        // Start initial timer in case there are no mutations at all
        timer = setTimeout(() => {
          observer.disconnect();
          clearTimeout(timeoutTimer);
          resolve({ success: true, stableMs: ${stableMs}, noMutations: true });
        }, ${stableMs});
      })`,
      awaitPromise: true,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error(`wait_for_stable: ${result.exceptionDetails.exception?.description || result.exceptionDetails.text}`);
    return result.result.value;
  },

  /**
   * is_actionable — 检查元素是否可操作（对齐 Kimi 7 项检查）
   * params: { selector }
   * 返回每项检查结果
   */
  async is_actionable(params) {
    const { selector } = params;
    if (!selector) throw new Error('is_actionable: selector is required');
    await ensureAttached((await resolveTab()).id);

    const result = await cdpCommand('Runtime.evaluate', {
      expression: `(() => {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) return { actionable: false, error: 'element not found' };
        const rect = el.getBoundingClientRect();
        const style = getComputedStyle(el);
        return {
          actionable: rect.width > 0 && rect.height > 0 && style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0',
          checks: {
            exists: true,
            visible: style.display !== 'none' && style.visibility !== 'hidden',
            hasSize: rect.width > 0 && rect.height > 0,
            inViewport: rect.top < window.innerHeight && rect.bottom > 0 && rect.left < window.innerWidth && rect.right > 0,
            notDisabled: !el.disabled,
            notPointerEventsNone: style.pointerEvents !== 'none',
            opacity: parseFloat(style.opacity) > 0,
          },
          rect: { x: Math.round(rect.x), y: Math.round(rect.y), width: Math.round(rect.width), height: Math.round(rect.height) },
          tag: el.tagName,
          text: (el.textContent || '').slice(0, 80),
        };
      })()`,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error(`is_actionable: ${result.exceptionDetails.text}`);
    return result.result.value;
  },

  /**
   * highlight — 高亮标记元素（调试用，对齐 Kimi）
   * params: { selector, duration?, color? }
   */
  async highlight(params) {
    const { selector, duration = 2000, color = '#00B4D8' } = params;
    if (!selector) throw new Error('highlight: selector is required');
    await ensureAttached((await resolveTab()).id);

    const result = await cdpCommand('Runtime.evaluate', {
      expression: `(() => {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) return { error: 'element not found' };
        const rect = el.getBoundingClientRect();
        const overlay = document.createElement('div');
        overlay.style.cssText = \`position:fixed;left:\${rect.left}px;top:\${rect.top}px;width:\${rect.width}px;height:\${rect.height}px;border:3px solid ${color};background:${color}22;z-index:2147483647;pointer-events:none;transition:opacity 0.3s;\`;
        document.body.appendChild(overlay);
        setTimeout(() => { overlay.style.opacity = '0'; setTimeout(() => overlay.remove(), 300); }, ${duration});
        return { success: true, selector: ${JSON.stringify(selector)}, rect: { x: Math.round(rect.x), y: Math.round(rect.y), w: Math.round(rect.width), h: Math.round(rect.height) } };
      })()`,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error(`highlight: ${result.exceptionDetails.text}`);
    const val = result.result.value;
    if (val?.error) throw new Error(`highlight: ${val.error}`);
    return val;
  },

  /**
   * query_elements — 查询页面元素（对齐 Kimi）
   * params: { selector?, role?, text?, limit? }
   * 返回匹配元素列表（tag, text, rect, attributes）
   */
  async query_elements(params) {
    const { selector, role, text, limit = 20 } = params;
    if (!selector && !role && !text) throw new Error('query_elements: need selector, role, or text');
    await ensureAttached((await resolveTab()).id);

    let expression;
    if (selector) {
      expression = `(() => {
        const els = [...document.querySelectorAll(${JSON.stringify(selector)})].slice(0, ${limit});
        return els.map(el => {
          const r = el.getBoundingClientRect();
          return { tag: el.tagName, text: (el.textContent||'').trim().slice(0,120), rect: {x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)}, id: el.id || undefined, className: el.className?.toString()?.slice(0,80) || undefined };
        });
      })()`;
    } else if (role) {
      expression = `(() => {
        const els = [...document.querySelectorAll('[role=${JSON.stringify(role)}]')].slice(0, ${limit});
        return els.map(el => {
          const r = el.getBoundingClientRect();
          return { tag: el.tagName, role: el.getAttribute('role'), text: (el.textContent||'').trim().slice(0,120), rect: {x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)} };
        });
      })()`;
    } else {
      expression = `(() => {
        const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
        const results = [];
        while (walker.nextNode() && results.length < ${limit}) {
          const node = walker.currentNode;
          if (node.textContent?.includes(${JSON.stringify(text)})) {
            const el = node.parentElement;
            if (el) {
              const r = el.getBoundingClientRect();
              results.push({ tag: el.tagName, text: (el.textContent||'').trim().slice(0,120), rect: {x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)} });
            }
          }
        }
        return results;
      })()`;
    }
    const result = await cdpCommand('Runtime.evaluate', { expression, returnByValue: true });
    if (result.exceptionDetails) throw new Error(`query_elements: ${result.exceptionDetails.text}`);
    return { count: result.result.value?.length || 0, elements: result.result.value || [] };
  },

  /**
   * trace — 操作追踪（记录操作历史，对齐 Kimi）
   * params: { action: "start" | "stop" | "get" }
   */
  async trace(params) {
    const cmd = params?.action || 'get';
    if (cmd === 'start') {
      _traceEnabled = true;
      _traceLog = [];
      return { success: true, message: 'trace started' };
    }
    if (cmd === 'stop') {
      _traceEnabled = false;
      return { success: true, trace: _traceLog };
    }
    return { trace: _traceLog, enabled: _traceEnabled };
  },
};

// ═══════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// URL 转 glob pattern（对齐 Kimi 的 find_tab 效率优化）
function urlToGlobPattern(url) {
  if (url.includes('*')) return url;
  try {
    return `*://${new URL(url).hostname}/*`;
  } catch {
    return `*://${url.replace(/^\.+/, '')}/*`;
  }
}

// ═══════════════════════════════════════════════
// Tab 分组（对齐 Kimi WebBridge）
// ═══════════════════════════════════════════════

async function assignToSession(tabId, session, groupTitle) {
  if (!session) return;
  try {
    const existingGroupId = sessionGroups.get(session);
    if (existingGroupId != null) {
      await chrome.tabs.group({ tabIds: [tabId], groupId: existingGroupId });
      return;
    }
    const groupName = `agent:${session}`;
    const existing = await chrome.tabGroups.query({ title: groupName });
    if (existing.length > 0) {
      await chrome.tabs.group({ tabIds: [tabId], groupId: existing[0].id });
      sessionGroups.set(session, existing[0].id);
      return;
    }
    const title = groupTitle || groupName;
    if (!sessionColors.has(session)) {
      // 优先用站点预定义颜色，其次轮询
      const siteColor = SITE_COLORS[session] || Object.entries(SITE_COLORS).find(([k]) => session.includes(k))?.[1];
      sessionColors.set(session, siteColor || SESSION_COLORS[sessionColorIdx++ % SESSION_COLORS.length]);
    }
    const groupId = await chrome.tabs.group({ tabIds: [tabId] });
    const color = sessionColors.get(session);
    await chrome.tabGroups.update(groupId, { title, color, collapsed: false });
    sessionGroups.set(session, groupId);
  } catch (e) {
    console.warn('[WebBridge] Tab group error:', e.message);
  }
}

// ═══════════════════════════════════════════════
// Network 捕获（对齐 Kimi WebBridge）
// ═══════════════════════════════════════════════

function ensureNetworkListener() {
  if (networkListenerAdded) return;
  networkListenerAdded = true;
  chrome.debugger.onEvent.addListener((source, method, params) => {
    const tabId = source.tabId;
    if (!tabId || !activeCaptures.has(tabId)) return;
    const captures = networkCaptures.get(tabId);
    if (!captures) return;

    if (method === 'Network.requestWillBeSent') {
      // 防止内存泄漏：每 tab 最多 1000 条
      if (captures.size >= 1000) {
        const oldest = captures.keys().next().value;
        captures.delete(oldest);
      }
      captures.set(params.requestId, {
        requestId: params.requestId,
        url: params.request.url,
        method: params.request.method,
        timestamp: params.timestamp,
      });
    }
    if (method === 'Network.responseReceived') {
      const req = captures.get(params.requestId);
      if (req) {
        req.status = params.response.status;
        req.mimeType = params.response.mimeType;
      }
    }
    if (method === 'Network.loadingFinished') {
      const req = captures.get(params.requestId);
      if (req) req.completed = true;
    }
  });
}

async function waitForLoad(tabId, timeout = 30000) {
  return new Promise((resolve) => {
    const listener = (id, info) => {
      if (id === tabId && info.status === 'complete') {
        chrome.tabs.onUpdated.removeListener(listener);
        resolve();
      }
    };
    chrome.tabs.onUpdated.addListener(listener);
    setTimeout(() => {
      chrome.tabs.onUpdated.removeListener(listener);
      resolve();
    }, timeout);
  });
}

function buildAxTree(nodes) {
  const nodeMap = new Map();
  for (const n of nodes) nodeMap.set(n.nodeId, n);
  
  function formatNode(node) {
    const role = typeof node.role === 'object' ? node.role?.value : node.role;
    
    if (!role || role === 'none' || role === 'generic') {
      const children = [];
      for (const cid of (node.childIds || [])) {
        const child = nodeMap.get(cid);
        if (child) {
          const result = formatNode(child);
          if (result) {
            if (Array.isArray(result)) children.push(...result);
            else children.push(result);
          }
        }
      }
      return children.length === 1 ? children[0] : children.length > 0 ? children : null;
    }
    
    const entry = { role };
    const name = typeof node.name === 'object' ? node.name?.value : node.name;
    if (name) entry.name = name;
    const value = typeof node.value === 'object' ? node.value?.value : node.value;
    if (value) entry.value = value;
    const desc = typeof node.description === 'object' ? node.description?.value : node.description;
    if (desc) entry.description = desc;
    
    if (REF_ROLES.has(role) && node.backendDOMNodeId != null) {
      entry.ref = `@${makeRef(node.backendDOMNodeId, role, name || '')}`;
    }
    
    const children = [];
    for (const cid of (node.childIds || [])) {
      const child = nodeMap.get(cid);
      if (child) {
        const result = formatNode(child);
        if (result) {
          if (Array.isArray(result)) children.push(...result);
          else children.push(result);
        }
      }
    }
    if (children.length > 0) entry.children = children;
    
    return entry;
  }
  
  if (nodes.length === 0) return [];
  const result = formatNode(nodes[0]);
  return result ? (Array.isArray(result) ? result : [result]) : [];
}

async function objectIdFromRef(ref) {
  const info = resolveRef(ref);
  if (!info) throw new Error(`Unknown ref "${ref}". Run snapshot first.`);
  const { object } = await cdpCommand('DOM.resolveNode', {
    backendNodeId: info.backendDOMNodeId,
  });
  if (!object?.objectId) throw new Error(`Could not resolve ref "${ref}"`);
  return object.objectId;
}

async function objectIdFromSelector(selector) {
  const result = await cdpCommand('Runtime.evaluate', {
    expression: `document.querySelector(${JSON.stringify(selector)})`,
    returnByValue: false,
  });
  if (result.result?.subtype === 'null') {
    throw new Error(`Element not found: ${selector}`);
  }
  return result.result.objectId;
}

async function clickByRef(ref) {
  const objectId = await objectIdFromRef(ref);
  const info = resolveRef(ref);
  
  await cdpCommand('Runtime.callFunctionOn', {
    objectId,
    functionDeclaration: "function() { this.scrollIntoView({block:'center',inline:'center'}); }",
  });
  
  const box = await cdpCommand('DOM.getBoxModel', { objectId });
  const c = box.model?.content;
  if (!c || c.length < 8) throw new Error(`Ref "${ref}" has no visible area`);
  
  const cx = (c[0] + c[2] + c[4] + c[6]) / 4;
  const cy = (c[1] + c[3] + c[5] + c[7]) / 4;
  
  await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseMoved', x: cx, y: cy, button: 'none', buttons: 0 });
  await cdpCommand('Input.dispatchMouseEvent', { type: 'mousePressed', x: cx, y: cy, button: 'left', buttons: 1, clickCount: 1 });
  await cdpCommand('Input.dispatchMouseEvent', { type: 'mouseReleased', x: cx, y: cy, button: 'left', buttons: 0, clickCount: 1 });
  
  return { success: true, ref, role: info.role, name: info.name, x: Math.round(cx), y: Math.round(cy) };
}

async function clickBySelector(selector) {
  // 用 JSON.stringify 安全转义 selector，防止注入
  const result = await cdpCommand('Runtime.evaluate', {
    expression: `(() => {
      const sel = ${JSON.stringify(selector)};
      const el = document.querySelector(sel);
      if (!el) return { error: 'element not found: ' + sel };
      el.scrollIntoView({ block: 'center' });
      el.click();
      return { success: true, tag: el.tagName, text: el.textContent?.slice(0, 100) };
    })()`,
    returnByValue: true,
  });
  if (result.exceptionDetails) throw new Error(`click: ${result.exceptionDetails.text}`);
  const value = result.result.value;
  if (value?.error) throw new Error(value.error);
  return value || { success: true };
}

async function fillByRef(ref, value) {
  const objectId = await objectIdFromRef(ref);
  
  const jsCode = `function() {
    const el = this;
    el.focus();
    if (el.isContentEditable) {
      const sel = window.getSelection();
      if (sel) {
        const range = document.createRange();
        range.selectNodeContents(el);
        sel.removeAllRanges();
        sel.addRange(range);
      }
      let inserted = false;
      try { inserted = document.execCommand('insertText', false, ${JSON.stringify(value)}); } catch(_e) { inserted = false; }
      if (!inserted) {
        el.textContent = ${JSON.stringify(value)};
        el.dispatchEvent(new InputEvent('input', {inputType:'insertText', data:${JSON.stringify(value)}, bubbles:true}));
      }
      return {success:true, tag:el.tagName, mode:'contenteditable'};
    }
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')?.set
      || Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value')?.set;
    if (setter) setter.call(el, ${JSON.stringify(value)});
    else el.value = ${JSON.stringify(value)};
    el.dispatchEvent(new Event('input', {bubbles:true}));
    el.dispatchEvent(new Event('change', {bubbles:true}));
    return {success:true, tag:el.tagName, mode:'value'};
  }`;
  
  const result = await cdpCommand('Runtime.callFunctionOn', {
    objectId,
    functionDeclaration: jsCode,
    returnByValue: true,
  });
  
  if (result.exceptionDetails) {
    throw new Error(`fill: ${result.exceptionDetails.text}`);
  }
  return result.result.value || { success: true };
}

async function fillBySelector(selector, value) {
  // 用 JSON.stringify 安全转义 selector 和 value，防止注入
  const result = await cdpCommand('Runtime.evaluate', {
    expression: `(() => {
      const sel = ${JSON.stringify(selector)};
      const val = ${JSON.stringify(value)};
      const el = document.querySelector(sel);
      if (!el) return { error: 'element not found: ' + sel };
      if (el.isContentEditable) {
        el.focus();
        const sel2 = window.getSelection();
        if (sel2) {
          const range = document.createRange();
          range.selectNodeContents(el);
          sel2.removeAllRanges();
          sel2.addRange(range);
        }
        let ok = false;
        try { ok = document.execCommand('insertText', false, val); } catch(_e) { ok = false; }
        if (!ok) {
          el.textContent = val;
          el.dispatchEvent(new InputEvent('input', {inputType:'insertText', data:val, bubbles:true}));
        }
        return { success: true, tag: el.tagName, mode: 'contenteditable' };
      }
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')?.set
        || Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value')?.set;
      if (setter) setter.call(el, val);
      else el.value = val;
      el.dispatchEvent(new Event('input', { bubbles: true }));
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return { success: true, tag: el.tagName, mode: 'value' };
    })()`,
    returnByValue: true,
    awaitPromise: false,
  });
  if (result.exceptionDetails) throw new Error(`fill: ${result.exceptionDetails.text}`);
  const val = result.result.value;
  if (val?.error) throw new Error(val.error);
  return val || { success: true };
}

// ═══════════════════════════════════════════════
// 事件监听
// ═══════════════════════════════════════════════

chrome.tabs.onRemoved.addListener((tabId) => {
  attachedTabs.delete(tabId);
  if (activeTabId === tabId) activeTabId = null;
  if (lastFocusedTabId === tabId) lastFocusedTabId = null;
  // 清理 network 捕获
  networkCaptures.delete(tabId);
  activeCaptures.delete(tabId);
});

chrome.debugger.onDetach.addListener((source) => {
  if (source.tabId) {
    attachedTabs.delete(source.tabId);
    if (activeTabId === source.tabId) activeTabId = null;
  }
});

chrome.tabs.onActivated.addListener((info) => {
  lastFocusedTabId = info.tabId;
});

// Tab 分组清理（对齐 Kimi）
chrome.tabGroups?.onRemoved?.addListener((group) => {
  for (const [session, gid] of sessionGroups) {
    if (gid === group.id) {
      sessionGroups.delete(session);
      break;
    }
  }
});

// Alarms 兜底重连（对齐 Kimi）
chrome.alarms?.onAlarm?.addListener((alarm) => {
  if (alarm.name === 'webbridge-reconnect' && !wsConnected) {
    connectWebSocket();
  }
});

// CDP 事件监听：自动处理 JS 对话框
chrome.debugger.onEvent.addListener((source, method, params) => {
  if (method === 'Page.javascriptDialogOpening') {
    const accept = _dialogAction === 'accept';
    const promptText = _dialogPromptText || undefined;
    chrome.debugger.sendCommand({ tabId: source.tabId }, 'Page.handleJavaScriptDialog', {
      accept,
      promptText,
    }).catch(() => {});
  }
});

// ═══════════════════════════════════════════════
// 消息处理（popup 查询状态）
// ═══════════════════════════════════════════════

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
    if (msg.type === 'getStatus') {
        sendResponse({
            connected: wsConnected,
            port: WS_PORT
        });
    }
    if (msg.type === 'getDetailedStatus') {
        (async () => {
            const tabs = await chrome.tabs.query({});
            const result = {
                connected: wsConnected,
                port: WS_PORT,
                version: '1.5.0',
                activeTabId,
                attachedTabs: [...attachedTabs],
                sessions: [...sessionGroups.keys()],
                tabCount: tabs.length,
                tabs: tabs.slice(0, 20).map(t => ({
                    id: t.id,
                    url: (t.url || '').slice(0, 60),
                    title: (t.title || '').slice(0, 40),
                    active: t.active,
                })),
            };
            sendResponse(result);
        })();
        return true; // async response
    }
    return true;
});

// ═══════════════════════════════════════════════
// 启动
// ═══════════════════════════════════════════════

console.log('[WebBridge] Background loaded');
loadPortConfig().then(() => connectWebSocket());
// Keepalive: 每 25s 检查连接状态，断线则重连
setInterval(() => {
  if (!wsConnected || ws?.readyState !== WebSocket.OPEN) {
    connectWebSocket();
  }
}, 25000);
