const DEFAULT_PORT = 19529;
const portInput = document.getElementById('port');
const connDot = document.getElementById('connDot');
const connLabel = document.getElementById('connLabel');
const statusBody = document.getElementById('statusBody');
const tabBody = document.getElementById('tabBody');

// ── 加载配置 ──
chrome.storage.sync.get(['wfPort'], (r) => {
    portInput.value = r.wfPort || DEFAULT_PORT;
});
// 启动时自动刷新
setTimeout(refresh, 100);

// ── 保存端口 ──
document.getElementById('save').addEventListener('click', () => {
    const port = parseInt(portInput.value, 10);
    if (port < 1024 || port > 65535) {
        connLabel.textContent = '无效端口';
        return;
    }
    chrome.storage.sync.set({ wfPort: port }, () => {
        connLabel.textContent = `已保存 :${port}`;
        setTimeout(refresh, 500);
    });
});

// ── 手动刷新 ──
document.getElementById('refresh').addEventListener('click', refresh);

// ── 刷新状态 ──
function refresh() {
    chrome.runtime.sendMessage({ type: 'getDetailedStatus' }, (s) => {
        if (!s) {
            setConn(false, '无响应');
            statusBody.innerHTML = '<div class="empty">无法连接后台</div>';
            tabBody.innerHTML = '<div class="empty">无法获取标签</div>';
            return;
        }

        // 连接状态
        setConn(s.connected, s.connected ? `已连接 :${s.port}` : `未连接 :${s.port}`);

        // 运行状态
        const attached = s.attachedTabs || [];
        const sessions = s.sessions || [];
        statusBody.innerHTML = [
            `<div class="info-row"><span class="label">版本</span><span>v${s.version || '?'}</span></div>`,
            `<div class="info-row"><span class="label">活跃标签</span><span>${s.activeTabId || '—'}</span></div>`,
            `<div class="info-row"><span class="label">CDP 附加</span><span>${attached.length} 个标签</span></div>`,
            `<div class="info-row"><span class="label">会话数</span><span>${sessions.length}${sessions.length ? ': ' + sessions.join(', ') : ''}</span></div>`,
        ].join('');

        // 标签列表
        if (s.tabs && s.tabs.length > 0) {
            tabBody.innerHTML = s.tabs.map(t => {
                const activeTag = t.active ? '<span class="tag tag-active">活跃</span>' : '';
                const url = t.url || '(无)';
                return `<div class="tab-item">${activeTag}<span class="url">${esc(url)}</span><span class="title">${esc(t.title || '')}</span></div>`;
            }).join('');
        } else {
            tabBody.innerHTML = '<div class="empty">无标签页</div>';
        }
    });
}

function setConn(ok, text) {
    connDot.className = ok ? 'dot dot-ok' : 'dot dot-err';
    connLabel.textContent = text;
}

function esc(s) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
