// Doc2Flow Generic Checklist Logic

const DOC_ID = window.D2F_DOC_ID || '';
const rawFilename = window.location.pathname.split('/').pop() || 'index.html';
const FILENAME = decodeURIComponent(rawFilename);
const STATE_KEY = 'd2f_state_' + (DOC_ID ? (DOC_ID + '_') : '') + FILENAME;

function handleLightboxKeydown(e) {
    if (e.key === 'Escape') {
        closeLightbox();
    }
}

function openLightbox(imgSrc) {
    const lbImg = document.getElementById('lb-img');
    const lightbox = document.getElementById('lightbox');
    if (lbImg && lightbox) {
        lbImg.src = imgSrc;
        lightbox.classList.add('active');
        document.addEventListener('keydown', handleLightboxKeydown);
    }
}

function closeLightbox() {
    const lightbox = document.getElementById('lightbox');
    if (lightbox) {
        lightbox.classList.remove('active');
    }
    document.removeEventListener('keydown', handleLightboxKeydown);
}

function updateEmptySections() {
    document.querySelectorAll('.section').forEach(sec => {
        const sh = sec.querySelector('.sh');
        const body = sec.querySelector('.sb');
        if (sh && body) {
            const isEmpty = body.children.length === 0 && body.innerHTML.trim() === '';
            if (isEmpty) {
                sh.classList.add('no-toggle');
                sh.removeAttribute('role');
                sh.removeAttribute('tabindex');
                sh.removeAttribute('aria-expanded');
            }
        }
    });
}

function toggleSection(headerElement) {
    if (typeof headerElement === 'string') {
        const sec = document.getElementById(headerElement);
        headerElement = sec ? sec.querySelector('.sh') : null;
    }
    if (!headerElement || (headerElement.classList && headerElement.classList.contains('no-toggle'))) return;
    const section = headerElement.closest('.section');
    const body = section ? section.querySelector('.sb') : null;
    
    if (body && (body.children.length > 0 || body.innerHTML.trim() !== '')) {
        const isCollapsed = body.classList.toggle('collapsed');
        headerElement.setAttribute('aria-expanded', isCollapsed ? 'false' : 'true');
        const toggler = headerElement.querySelector('.stog');
        if (toggler) {
            toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
        }
    }
}

function autoExpandTextarea(el) {
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = el.scrollHeight + 'px';
}

function getOrCreateCommentBox(checkItem, initialValue) {
    if (!checkItem) return null;
    let box = checkItem.querySelector('.item-comment-box');
    let input;
    if (!box) {
        box = document.createElement('div');
        box.className = 'item-comment-box';
        
        input = document.createElement('textarea');
        input.rows = 1;
        input.className = 'item-comment-input';
        const i18n = window.D2F_I18N || {};
        const commentLabel = i18n.comment_placeholder || 'Add a comment...';
        input.placeholder = commentLabel;
        input.setAttribute('aria-label', commentLabel);
        
        const delBtn = document.createElement('button');
        delBtn.type = 'button';
        delBtn.className = 'item-comment-del';
        delBtn.title = 'Delete comment';
        delBtn.setAttribute('aria-label', 'Delete comment');
        delBtn.innerHTML = '&#10006;';
        
        box.appendChild(input);
        box.appendChild(delBtn);
        checkItem.appendChild(box);
    } else {
        input = box.querySelector('.item-comment-input');
    }
    
    if (typeof initialValue === 'string') {
        input.value = initialValue;
        input.textContent = initialValue;
        input.setAttribute('value', initialValue);
    }
    
    autoExpandTextarea(input);

    return { box, input };
}

function saveState() {
    const state = {};
    document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
        const key = cb.id || ('cb_' + index);
        state[key] = cb.checked;
    });
    
    const textStates = {};
    document.querySelectorAll('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
        const key = item.id || ('txt_' + index);
        textStates[key] = item.classList.contains('checked');
    });

    const fields = {};
    document.querySelectorAll('input.persistent-field').forEach((input, index) => {
        const key = input.id || ('f_' + index);
        fields[key] = input.value;
    });

    const comments = {};
    document.querySelectorAll('.check-item').forEach((item, index) => {
        const input = item.querySelector('.item-comment-input');
        if (input && input.value.trim() !== '') {
            const key = item.id || ('item_' + index);
            comments[key] = input.value;
        }
    });

    try { 
        localStorage.setItem(STATE_KEY, JSON.stringify({ checks: state, texts: textStates, fields: fields, comments: comments })); 
    } catch (e) {
        console.warn('Failed to save state to localStorage', e);
    }
}

function loadState() {
    try {
        const raw = localStorage.getItem(STATE_KEY);
        if (!raw) return;
        
        const data = JSON.parse(raw);
        if (data.checks) {
            document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
                const key = cb.id || ('cb_' + index);
                if (data.checks[key] !== undefined) { 
                    cb.checked = data.checks[key]; 
                    styleItem(cb); 
                }
            });
        }
        if (data.texts) {
            document.querySelectorAll('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
                const key = item.id || ('txt_' + index);
                if (data.texts[key] !== undefined) {
                    item.classList.toggle('checked', data.texts[key]);
                }
            });
        }
        if (data.fields) {
            document.querySelectorAll('input.persistent-field').forEach((input, index) => {
                const key = input.id || ('f_' + index);
                if (data.fields[key] !== undefined) {
                    input.value = data.fields[key];
                }
            });
        }
        if (data.comments) {
            document.querySelectorAll('.check-item').forEach((item, index) => {
                const key = item.id || ('item_' + index);
                if (data.comments[key] !== undefined) {
                    getOrCreateCommentBox(item, data.comments[key]);
                }
            });
        }
        syncLinkedFields();
    } catch (e) {
        console.warn('Failed to load state from localStorage', e);
    }
}

function styleItem(cb) {
    const item = cb.closest('.check-item');
    if (item) {
        item.classList.toggle('checked', cb.checked);
    }
}

function updateProgress() {
    const i18n = window.D2F_I18N || {};
    const sections = document.querySelectorAll('.section');
    let total = 0, done = 0;

    // Batch read phase (eliminate layout thrashing - JS-PERF-ASYNC)
    const updates = Array.from(sections).map(sec => {
        const cbs = Array.from(sec.querySelectorAll('input[type="checkbox"]'));
        const badge = sec.querySelector('.sbadge');
        const count = cbs.length;
        const checkedCount = cbs.filter(c => c.checked).length;
        total += count;
        done += checkedCount;
        return { badge, count, checkedCount };
    });

    // Batch write phase
    updates.forEach(({ badge, count, checkedCount }) => {
        if (!badge) return;
        if (count === 0) {
            badge.textContent = '';
            badge.style.display = 'none';
        } else {
            badge.style.display = '';
            badge.textContent = checkedCount + ' / ' + count;
            badge.className = 'sbadge' + (checkedCount === count ? ' done' : '');
        }
    });

    const pct = total ? Math.round((done / total) * 100) : 0;
    const pb = document.getElementById('pb');
    if (pb) {
        pb.style.width = pct + '%';
        if (pb.parentElement) {
            pb.parentElement.setAttribute('aria-valuenow', pct);
        }
    }

    const pt = document.getElementById('pt');
    if (pt) {
        const tmpl = i18n.progress_template || '{done} of {total} tasks completed ({pct}%)';
        pt.textContent = tmpl.replace('{done}', done).replace('{total}', total).replace('{pct}', pct);
    }

    const finishBox = document.getElementById('finish-box');
    const finishIcon = document.getElementById('finish-icon');
    const finishTitle = document.getElementById('finish-title');
    const btnPdf = document.getElementById('btn-pdf');

    if (finishBox) {
        finishBox.classList.remove('completed', 'pending', 'no-tasks');
        if (total === 0) {
            finishBox.classList.add('no-tasks');
            if (btnPdf) btnPdf.disabled = false;
        } else if (done < total) {
            finishBox.classList.add('pending');
            if (finishIcon) finishIcon.innerHTML = '&#x29D6;';
            if (finishTitle) finishTitle.textContent = i18n.setup_in_progress || 'Setup in Progress';
            if (btnPdf) btnPdf.disabled = true;
        } else {
            finishBox.classList.add('completed');
            if (finishIcon) finishIcon.innerHTML = '&#x2714;';
            if (finishTitle) finishTitle.textContent = i18n.setup_completed || 'Setup Completed';
            if (btnPdf) btnPdf.disabled = false;
        }
    }
}

function resetAll() {
    const i18n = window.D2F_I18N || {};
    const msg = i18n.confirm_reset || 'Are you sure you want to reset all markings?';
    if (!confirm(msg)) return;
    document.querySelectorAll('.check-item input[type="checkbox"]').forEach(c => { 
        c.checked = false; 
        styleItem(c); 
    });
    document.querySelectorAll('.check-item.text-item, .check-item.simple-item').forEach(item => {
        item.classList.remove('checked');
    });
    updateProgress(); 
    saveState();
}

// Saves current state into DOM attributes and downloads the updated HTML file (JS-STATE-DOM-SYNC)
function saveDocumentState() {
    saveState();

    document.querySelectorAll('.check-item input[type="checkbox"]').forEach(cb => {
        if (cb.checked) {
            cb.setAttribute('checked', 'checked');
        } else {
            cb.removeAttribute('checked');
        }
        styleItem(cb);
    });

    document.querySelectorAll('input:not([type="checkbox"]):not([type="radio"])').forEach(input => {
        input.setAttribute('value', input.value);
    });

    document.querySelectorAll('textarea').forEach(ta => {
        ta.textContent = ta.value;
        ta.setAttribute('value', ta.value);
    });

    document.querySelectorAll('select').forEach(select => {
        Array.from(select.options).forEach(opt => {
            if (opt.selected) {
                opt.setAttribute('selected', 'selected');
            } else {
                opt.removeAttribute('selected');
            }
        });
    });

    const htmlContent = '<!DOCTYPE html>\n' + document.documentElement.outerHTML;
    const blob = new Blob([htmlContent], { type: 'text/html;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = FILENAME;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

// Export the current state as PDF via browser print.
function exportPDF() {
    const btnPdf = document.getElementById('btn-pdf');
    if (btnPdf && btnPdf.disabled) return;

    const collapsed = Array.from(document.querySelectorAll('.sb.collapsed'));
    collapsed.forEach(el => el.classList.remove('collapsed'));
    
    document.querySelectorAll('textarea.item-comment-input').forEach(ta => {
        ta.textContent = ta.value;
        ta.setAttribute('value', ta.value);
        autoExpandTextarea(ta);
    });

    const restore = () => {
        collapsed.forEach(el => el.classList.add('collapsed'));
        window.removeEventListener('afterprint', restore);
    };
    
    window.addEventListener('afterprint', restore);
    setTimeout(() => window.print(), 100);
}

function syncFieldPair(id1, id2, sourceInput) {
    const el1 = document.getElementById(id1);
    const el2 = document.getElementById(id2);
    if (!el1 || !el2) return;

    if (sourceInput === el1) {
        el2.value = el1.value;
    } else if (sourceInput === el2) {
        el1.value = el2.value;
    } else {
        if (el1.value && !el2.value) {
            el2.value = el1.value;
        } else if (el2.value && !el1.value) {
            el1.value = el2.value;
        } else if (el1.value) {
            el2.value = el1.value;
        }
    }
}

function syncLinkedFields(sourceInput) {
    syncFieldPair('f_info_agent', 'f_sign_agent', sourceInput);
    syncFieldPair('f_info_date', 'f_sign_date', sourceInput);
}

function formatDateFromTemplate(now, template) {
    if (!template || typeof template !== 'string') return null;

    const day = String(now.getDate()).padStart(2, '0');
    const dayShort = String(now.getDate());
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const monthShort = String(now.getMonth() + 1);
    const yearFull = String(now.getFullYear());
    const yearShort = String(now.getFullYear()).slice(-2);

    const tokenMap = {
        'YYYY': yearFull,
        'JJJJ': yearFull,
        'YY': yearShort,
        'JJ': yearShort,
        'MM': month,
        'DD': day,
        'TT': day,
        'M': monthShort,
        'D': dayShort,
        'T': dayShort,
        'Y': yearFull,
        'J': yearFull
    };

    const regex = /YYYY|JJJJ|YY|JJ|MM|DD|TT|M|D|T|Y|J/gi;
    let hasMatches = false;

    const formatted = template.replace(regex, (match) => {
        hasMatches = true;
        const upperMatch = match.toUpperCase();
        return tokenMap[upperMatch] || match;
    });

    if (!hasMatches || /[A-Za-z]/.test(formatted)) {
        return null;
    }

    return formatted;
}

function getTodayFormatted() {
    const i18n = window.D2F_I18N || {};
    const now = new Date();
    const placeholder = i18n.date_placeholder;

    try {
        const fromTemplate = formatDateFromTemplate(now, placeholder);
        if (fromTemplate) {
            return fromTemplate;
        }
    } catch (e) {
        console.warn('Failed to format date from date_placeholder template', e);
    }

    return now.toLocaleDateString(navigator.language || undefined);
}

function checkDateShortcut(input) {
    if (!input || typeof input.value !== 'string') return false;
    const val = input.value.trim().toLowerCase();
    if (val === 'today') {
        input.value = getTodayFormatted();
        return true;
    }
    return false;
}

function copyCode(btn) {
    const wrap = btn.closest('.code-block-wrap');
    if (!wrap) return;
    const codeEl = wrap.querySelector('code');
    if (!codeEl) return;

    const text = codeEl.innerText || codeEl.textContent;
    if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
        navigator.clipboard.writeText(text).then(() => {
            showCopiedFeedback(btn);
        }).catch(() => fallbackCopyText(text, btn));
    } else {
        fallbackCopyText(text, btn);
    }
}

function fallbackCopyText(text, btn) {
    try {
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        ta.style.top = '-9999px';
        ta.style.opacity = '0';
        ta.setAttribute('readonly', '');
        document.body.appendChild(ta);
        ta.select();
        
        const successful = typeof document.execCommand === 'function' ? document.execCommand('copy') : false;
        document.body.removeChild(ta);
        if (successful) {
            showCopiedFeedback(btn);
        }
    } catch (e) {
        console.warn('Fallback copy failed:', e);
    }
}

function showCopiedFeedback(btn) {
    btn.classList.add('copied');
    setTimeout(() => btn.classList.remove('copied'), 2000);
}

// Global Event Delegation & Initialization (JS-EVENT-LIFECYCLE & JS-DOM-EFFICIENT)
document.addEventListener('DOMContentLoaded', () => {
    // 1. Delegated click & keydown listeners for interactive items, headers, and images
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ' || e.key === 'Spacebar') {
            const sh = e.target.closest('.sh');
            if (sh && !sh.classList.contains('no-toggle')) {
                e.preventDefault();
                toggleSection(sh);
            }
        }
    });

    document.addEventListener('click', (e) => {
        const img = e.target.closest('.doc-body img');
        if (img) {
            e.stopPropagation();
            openLightbox(img.src);
            return;
        }

        const sh = e.target.closest('.sh');
        if (sh && !sh.classList.contains('no-toggle')) {
            toggleSection(sh);
            return;
        }

        const commentBtn = e.target.closest('.item-comment-icon');
        if (commentBtn) {
            const checkItem = commentBtn.closest('.check-item');
            if (checkItem) {
                const res = getOrCreateCommentBox(checkItem);
                if (res && res.input) {
                    res.input.focus();
                }
            }
            return;
        }

        const commentDelBtn = e.target.closest('.item-comment-del');
        if (commentDelBtn) {
            const box = commentDelBtn.closest('.item-comment-box');
            if (box) {
                box.remove();
                saveState();
            }
            return;
        }

        const checkItem = e.target.closest('.check-item');
        if (checkItem) {
            if (e.target.tagName === 'A' || e.target.tagName === 'IMG' || e.target.closest('.item-comment-box')) return;

            const cb = checkItem.querySelector('input[type="checkbox"]');
            if (cb) {
                if (e.target !== cb && !e.target.closest('label')) {
                    cb.checked = !cb.checked;
                }
                styleItem(cb);
                updateProgress();
                saveState();
            } else if (checkItem.classList.contains('text-item') || checkItem.classList.contains('simple-item')) {
                checkItem.classList.toggle('checked');
                saveState();
            }
        }
    });

    document.addEventListener('change', (e) => {
        if (e.target && e.target.matches('.check-item input[type="checkbox"]')) {
            styleItem(e.target);
            updateProgress();
            saveState();
        }
    });

    // 2. Delegated input/change listeners for persistent fields, linked fields, and date shortcuts
    const linkedIds = ['f_info_agent', 'f_sign_agent', 'f_info_date', 'f_sign_date'];
    const handleInputOrChange = (e) => {
        const target = e.target;
        if (!target) return;

        if (target.classList.contains('persistent-field')) {
            saveState();
        }

        if (target.classList.contains('item-comment-input')) {
            target.textContent = target.value;
            target.setAttribute('value', target.value);
            autoExpandTextarea(target);
            saveState();
        }

        if (target.id && linkedIds.includes(target.id)) {
            if (target.id.toLowerCase().includes('date')) {
                checkDateShortcut(target);
            }
            syncLinkedFields(target);
            saveState();
        } else if (target.matches('input[id*="date"], input[name*="date"], input.date-field')) {
            checkDateShortcut(target);
            saveState();
        }
    };

    document.addEventListener('input', handleInputOrChange);
    document.addEventListener('change', handleInputOrChange);
    document.addEventListener('blur', (e) => {
        if (e.target && (linkedIds.includes(e.target.id) || e.target.matches('input[id*="date"], input[name*="date"], input.date-field'))) {
            checkDateShortcut(e.target);
            saveState();
        }
    }, true);

    window.addEventListener('beforeprint', () => {
        document.querySelectorAll('textarea.item-comment-input').forEach(ta => {
            ta.textContent = ta.value;
            ta.setAttribute('value', ta.value);
            autoExpandTextarea(ta);
        });
    });

    window.addEventListener('resize', () => {
        document.querySelectorAll('textarea.item-comment-input').forEach(autoExpandTextarea);
    });

    // Initialize document state
    updateEmptySections();
    loadState();
    syncLinkedFields();
    updateProgress();
});
