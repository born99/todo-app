const { invoke } = window.__TAURI__.core;

let allTasks = [];
let selectedPriority = 'medium';
let editingTaskId = null;

// ─── Bootstrap ─────────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', () => {
    loadTasks();
});


// ─── Tab Switching ──────────────────────────────────────────────
function switchTab(tabName, btn) {
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    document.querySelectorAll('.nav-item').forEach(b => b.classList.remove('active'));
    document.getElementById('tab-' + tabName).classList.add('active');
    btn.classList.add('active');
    renderAll();
}

// ─── Data Loading ───────────────────────────────────────────────
async function loadTasks() {
    try {
        allTasks = await invoke('get_tasks');
    } catch (e) {
        console.error('Failed to load tasks:', e);
        allTasks = [];
    }
    renderAll();
}

// ─── Rendering ──────────────────────────────────────────────────
function renderAll() {
    renderTaskList();
    renderCalendar();
    renderAnalytics();
}

function priorityBadge(p) {
    const map = {
        low: ['badge-low', '🟢 Low'],
        medium: ['badge-medium', '🟡 Medium'],
        high: ['badge-red', '🔴 High'],
    };
    const [cls, label] = map[p] || map['medium'];
    return `<span class="badge ${cls}">${label}</span>`;
}

function dateBadge(dateStr) {
    if (!dateStr) return '';
    const d = new Date(dateStr);
    const formatted = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    return `<span class="badge badge-date">⏰ ${formatted}</span>`;
}

function durationBadge(mins) {
    if (!mins) return '';
    return `<span class="badge badge-dur">⏳ ${mins}m</span>`;
}

function makeTaskCard(task, showDone = false) {
    const done = task.is_completed;
    if (done && !showDone) return '';
    return `
    <div class="task-card ${done ? 'done' : ''}" data-id="${task.id}">
      <div class="task-checkbox ${done ? 'checked' : ''}" onclick="toggleTask(${task.id}, ${done})">
        ${done ? '✓' : ''}
      </div>
      <div class="task-body">
        <div class="task-title ${done ? 'done-text' : ''}">${escHtml(task.title)}</div>
        ${task.description ? `<div class="task-desc">${escHtml(task.description)}</div>` : ''}
        <div class="task-meta">
          ${priorityBadge(task.priority)}
          ${dateBadge(task.due_date)}
          ${durationBadge(task.duration_minutes)}
        </div>
      </div>
      <div class="task-actions">
        <button class="icon-btn" onclick="openEditModal(${task.id})" title="Edit">✎</button>
        <button class="icon-btn" onclick="deleteTask(${task.id})" title="Delete">🗑</button>
      </div>
    </div>`;
}

function renderTaskList() {
    const container = document.getElementById('task-list');
    const empty = document.getElementById('empty-state');
    const pending = allTasks.filter(t => !t.is_completed);
    const completed = allTasks.filter(t => t.is_completed);

    if (pending.length === 0 && completed.length === 0) {
        container.innerHTML = '';
        empty.style.display = 'flex';
        return;
    }
    empty.style.display = 'none';

    let html = pending.map(t => makeTaskCard(t)).join('');
    if (completed.length > 0) {
        html += `<details style="margin-top:20px"><summary style="color:var(--text-3);cursor:pointer;font-size:13px;font-weight:600;padding:8px 0;">Completed (${completed.length})</summary>`;
        html += completed.map(t => makeTaskCard(t, true)).join('');
        html += '</details>';
    }
    container.innerHTML = html;
}

function renderCalendar() {
    const scheduled = allTasks.filter(t => !t.is_completed && t.due_date);
    const unscheduled = allTasks.filter(t => !t.is_completed && !t.due_date);

    document.getElementById('scheduled-list').innerHTML =
        scheduled.length ? scheduled.map(t => makeTaskCard(t)).join('') : `<p style="color:var(--text-3);padding:20px 0">No scheduled tasks.</p>`;
    document.getElementById('unscheduled-list').innerHTML =
        unscheduled.length ? unscheduled.map(t => makeTaskCard(t)).join('') : `<p style="color:var(--text-3);padding:20px 0">No unscheduled tasks.</p>`;
}

function renderAnalytics() {
    const total = allTasks.length;
    const completed = allTasks.filter(t => t.is_completed).length;
    const pending = total - completed;
    const pct = total > 0 ? Math.round((completed / total) * 100) : 0;

    document.getElementById('stat-total').textContent = total;
    document.getElementById('stat-done').textContent = completed;
    document.getElementById('stat-pending').textContent = pending;
    document.getElementById('stat-pct').textContent = pct + '%';
    document.getElementById('progress-fill').style.width = pct + '%';
    document.getElementById('progress-label').textContent = pct + '% complete';
}

// ─── Actions ────────────────────────────────────────────────────
async function toggleTask(id, wasDone) {
    if (wasDone) return; // Already done, clicking again does nothing
    try {
        await invoke('complete_task', { id });
        await loadTasks();
    } catch (e) { console.error(e); }
}

async function deleteTask(id) {
    try {
        await invoke('delete_task', { id });
        await loadTasks();
    } catch (e) { console.error(e); }
}

// ─── Modal ──────────────────────────────────────────────────────
function openAddModal() {
    editingTaskId = null;
    document.getElementById('modal-title-text').textContent = 'New Task';
    document.getElementById('modal-submit-btn').textContent = 'Create Task';
    document.getElementById('modal-overlay').classList.add('open');
    document.getElementById('f-title').focus();
}

function openEditModal(id) {
    const task = allTasks.find(t => t.id === id);
    if (!task) return;
    editingTaskId = id;

    document.getElementById('modal-title-text').textContent = 'Edit Task';
    document.getElementById('modal-submit-btn').textContent = 'Save Changes';

    document.getElementById('f-title').value = task.title;
    document.getElementById('f-desc').value = task.description || '';
    document.getElementById('f-duration').value = task.duration_minutes || '';

    if (task.due_date) {
        const dObj = new Date(task.due_date);
        const pad = n => String(n).padStart(2, '0');
        document.getElementById('f-due-datetime').value = `${dObj.getFullYear()}-${pad(dObj.getMonth() + 1)}-${pad(dObj.getDate())}T${pad(dObj.getHours())}:${pad(dObj.getMinutes())}`;
    } else {
        document.getElementById('f-due-datetime').value = '';
    }
    document.getElementById('f-alert-early').value = task.alert_early_minutes || '0';

    document.querySelectorAll('#priority-chips .chip').forEach(c => c.classList.remove('selected'));
    const pBtn = document.querySelector(`#priority-chips [data-value="${task.priority}"]`);
    if (pBtn) pBtn.classList.add('selected');
    selectedPriority = task.priority;

    document.getElementById('modal-overlay').classList.add('open');
}

function closeAddModal() {
    document.getElementById('modal-overlay').classList.remove('open');
    document.getElementById('f-title').value = '';
    document.getElementById('f-desc').value = '';
    document.getElementById('f-duration').value = '';
    document.getElementById('f-due-datetime').value = '';
    document.getElementById('f-alert-early').value = '0';
    selectedPriority = 'medium';
    document.querySelectorAll('#priority-chips .chip').forEach(c => c.classList.remove('selected'));
    document.querySelector('#priority-chips [data-value="medium"]').classList.add('selected');
}

function closeModal(event) {
    if (event.target === document.getElementById('modal-overlay')) closeAddModal();
}

function selectPriority(btn) {
    document.querySelectorAll('#priority-chips .chip').forEach(c => c.classList.remove('selected'));
    btn.classList.add('selected');
    selectedPriority = btn.dataset.value;
}

function setDueDate(option) {
    const dtInput = document.getElementById('f-due-datetime');
    if (!option) {
        dtInput.value = '';
        return;
    }

    const now = new Date();
    if (option === 'tomorrow') now.setDate(now.getDate() + 1);
    if (option === 'week') now.setDate(now.getDate() + 7);

    if (option === 'today') now.setHours((now.getHours() + 1) % 24, 0, 0, 0);
    else now.setHours(18, 0, 0, 0);

    const pad = n => String(n).padStart(2, '0');
    dtInput.value = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
}

async function handleSubmit(event) {
    event.preventDefault();
    const title = document.getElementById('f-title').value.trim();
    if (!title) return;

    const desc = document.getElementById('f-desc').value.trim() || null;
    const durVal = document.getElementById('f-duration').value.trim();
    const duration_minutes = durVal ? parseInt(durVal) : null;

    const dtVal = document.getElementById('f-due-datetime').value;
    let due_date = null;
    if (dtVal) {
        due_date = new Date(dtVal).toISOString();
    }
    const alert_early_minutes = parseInt(document.getElementById('f-alert-early').value) || 0;

    try {
        const payload = {
            title,
            description: desc,
            priority: selectedPriority,
            duration_minutes,
            due_date,
            alert_early_minutes,
        };

        if (editingTaskId) {
            await invoke('edit_task', { id: editingTaskId, payload });
        } else {
            await invoke('create_task', { payload });
        }

        closeAddModal();
        await loadTasks();
    } catch (e) {
        console.error('Failed to save task:', e);
        alert('Error: ' + e);
    }
}

// ─── Utils ──────────────────────────────────────────────────────
function escHtml(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
