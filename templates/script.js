// Doc2Flow Generic Checklist Logic

// Use a unique ID injected by the generator, fallback to pathname
const DOC_ID = window.D2F_DOC_ID || window.location.pathname.replace(/[^a-zA-Z0-9]/g, '_');
const STATE_KEY = 'd2f_state_' + DOC_ID;

function openLightbox(imgSrc) {
    const lbImg = document.getElementById('lb-img');
    const lightbox = document.getElementById('lightbox');
    if (lbImg && lightbox) {
        lbImg.src = imgSrc;
        lightbox.classList.add('active');
    }
}

function closeLightbox() {
    const lightbox = document.getElementById('lightbox');
    if (lightbox) lightbox.classList.remove('active');
}

document.addEventListener('keydown', e => { 
    if(e.key === 'Escape') closeLightbox(); 
});

function toggleSection(headerElement) {
    // Expect the next sibling to be the section body (.sb)
    let body = headerElement.nextElementSibling;
    while (body && !body.classList.contains('sb')) {
        body = body.nextElementSibling;
    }
    
    if (body) {
        const isCollapsed = body.classList.toggle('collapsed');
        const toggler = headerElement.querySelector('.stog');
        if (toggler) {
            toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
        }
    }
}

function saveState() {
    const state = {};
    document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
        // Use an index or an ID if available. d2f generator will try to assign unique IDs to checkboxes
        const key = cb.id || ('cb_' + index);
        state[key] = cb.checked;
    });
    
    const textStates = {};
    document.querySelectorAll('.check-item.text-item').forEach((item, index) => {
        const key = item.id || ('txt_' + index);
        textStates[key] = item.classList.contains('checked');
    });

    // Save any persistent inputs (e.g. signature fields)
    const fields = {};
    document.querySelectorAll('input.persistent-field').forEach((input, index) => {
        const key = input.id || ('f_' + index);
        fields[key] = input.value;
    });

    try { 
        localStorage.setItem(STATE_KEY, JSON.stringify({ checks: state, texts: textStates, fields: fields })); 
    } catch(e) {
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
            document.querySelectorAll('.check-item.text-item').forEach((item, index) => {
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
    } catch(e) {
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
    const sections = document.querySelectorAll('.section');
    let total = 0, done = 0;
    
    sections.forEach(sec => {
        const cbs = sec.querySelectorAll('input[type="checkbox"]');
        if (cbs.length === 0) return;
        
        const checkedCount = [...cbs].filter(c => c.checked).length;
        total += cbs.length;
        done += checkedCount;
        
        const badge = sec.querySelector('.sbadge');
        if (badge) { 
            badge.textContent = checkedCount + ' / ' + cbs.length; 
            badge.className = 'sbadge' + (checkedCount === cbs.length ? ' done' : ''); 
        }
    });
    
    const pct = total ? Math.round(done / total * 100) : 0;
    const pb = document.getElementById('pb');
    if (pb) pb.style.width = pct + '%';
    
    const pt = document.getElementById('pt');
    if (pt) {
        const i18n = window.D2F_I18N || {};
        const tmpl = i18n.progress_template || '{done} of {total} tasks completed ({pct}%)';
        pt.textContent = tmpl.replace('{done}', done).replace('{total}', total).replace('{pct}', pct);
    }

    const finishBox = document.getElementById('finish-box');
    const finishIcon = document.getElementById('finish-icon');
    const finishTitle = document.getElementById('finish-title');
    const btnPdf = document.getElementById('btn-pdf');
    const i18n = window.D2F_I18N || {};

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
    const msg = i18n.confirm_reset || 'Are you sure you want to reset all checkboxes?';
    if (!confirm(msg)) return;
    document.querySelectorAll('.check-item input[type="checkbox"]').forEach(c => { 
        c.checked = false; 
        styleItem(c); 
    });
    document.querySelectorAll('.check-item.text-item').forEach(item => {
        item.classList.remove('checked');
    });
    updateProgress(); 
    saveState();
}

// Export the current state as PDF via the browser's "Save as PDF" print option.
// Collapsed sections are temporarily expanded so nothing is hidden in the PDF.
function exportPDF() {
    const btnPdf = document.getElementById('btn-pdf');
    if (btnPdf && btnPdf.disabled) return;

    const collapsed = [...document.querySelectorAll('.sb.collapsed')];
    collapsed.forEach(el => el.classList.remove('collapsed'));
    
    const restore = () => {
        collapsed.forEach(el => el.classList.add('collapsed'));
        window.removeEventListener('afterprint', restore);
    };
    
    window.addEventListener('afterprint', restore);
    setTimeout(() => window.print(), 100);
}

// Setup Event Listeners
document.addEventListener('DOMContentLoaded', () => {
    // Bind checkbox and text item clicks
    document.querySelectorAll('.check-item').forEach(item => {
        item.addEventListener('click', function(e) {
            // Ignore if clicked on a link
            if (e.target.tagName === 'A') return;
            
            const cb = this.querySelector('input[type="checkbox"]');
            if (cb) {
                // If clicked on the wrapper but not the checkbox directly, toggle it manually
                if (e.target !== cb) cb.checked = !cb.checked;
                styleItem(cb);
                updateProgress();
                saveState();
            } else if (this.classList.contains('text-item')) {
                this.classList.toggle('checked');
                saveState();
            }
        });
    });

    // Bind section header clicks
    document.querySelectorAll('.sh').forEach(sh => {
        sh.addEventListener('click', function() {
            toggleSection(this);
        });
    });

    // Bind image lightbox clicks
    document.querySelectorAll('.screenshot-wrap img, .ss-wrap img, img.lightbox-trigger').forEach(img => {
        img.style.cursor = 'zoom-in';
        img.addEventListener('click', function(e) {
            e.stopPropagation();
            openLightbox(this.src);
        });
    });

    // Bind persistent fields
    document.querySelectorAll('input.persistent-field').forEach(input => {
        input.addEventListener('input', saveState);
    });

    // Initialize state
    loadState();
    updateProgress();
});

function copyCode(btn) {
    const wrap = btn.closest('.code-block-wrap');
    if (!wrap) return;
    const codeEl = wrap.querySelector('code');
    if (!codeEl) return;

    const text = codeEl.innerText || codeEl.textContent;
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(() => {
            showCopiedFeedback(btn);
        }).catch(() => fallbackCopyText(text, btn));
    } else {
        fallbackCopyText(text, btn);
    }
}

function fallbackCopyText(text, btn) {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    try {
        document.execCommand('copy');
        showCopiedFeedback(btn);
    } catch (e) {
        console.error('Fallback copy failed', e);
    }
    document.body.removeChild(ta);
}

function showCopiedFeedback(btn) {
    btn.classList.add('copied');
    setTimeout(() => btn.classList.remove('copied'), 2000);
}
