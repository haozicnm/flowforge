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
        connDot.className = 'dot dot-err';
        return;
    }
    chrome.storage.sync.set({ wfPort: port }, () => {
        connLabel.textContent = `已保存 :${port}`;
        connDot.className = 'dot dot-checking';
        setTimeout(refresh, 500);
    });
});

// ── 回车保存 ──
portInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') document.getElementById('save').click();
});

// ── 手动刷新 ──
document.getElementById('refresh').addEventListener('click', refresh);

// ── 刷新状态 ──
function refresh() {
    connDot.className = 'dot dot-checking';
    connLabel.textContent = '检查中...';

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

        statusBody.innerHTML = `
            <div class="info-grid">
                <div class="info-item">
                    <span class="label">版本</span>
                    <span class="value">v${esc(s.version || '?')}</span>
                </div>
                <div class="info-item">
                    <span class="label">活跃标签</span>
                    <span class="value">${s.activeTabId || '—'}</span>
                </div>
                <div class="info-item">
                    <span class="label">CDP 附加</span>
                    <span class="value">${attached.length} 个</span>
                </div>
                <div class="info-item">
                    <span class="label">会话数</span>
                    <span class="value">${sessions.length}</span>
                </div>
            </div>
            ${sessions.length ? `<div style="margin-top:6px;font-size:10px;color:var(--ff-text-disabled)">${sessions.map(esc).join(' · ')}</div>` : ''}
        `;

        // 标签列表
        if (s.tabs && s.tabs.length > 0) {
            tabBody.innerHTML = s.tabs.map(t => {
                const tag = t.active
                    ? '<span class="tag tag-active">● 活跃</span>'
                    : '';
                return `
                    <div class="tab-item">
                        ${tag}
                        <span class="url">${esc(t.url || '(无)')}</span>
                        ${t.title ? `<span class="title">${esc(t.title)}</span>` : ''}
                    </div>
                `;
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
    const div = document.createElement('div');
    div.textContent = s;
    return div.innerHTML;
}
