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

function toggleSection(headerElement) {
    const section = headerElement.closest('.section');
    const body = section ? section.querySelector('.sb') : null;
    
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
    document.querySelectorAll('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
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
        syncLinkedFields();
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
    const i18n = window.D2F_I18N || {};
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

// Saves current state into DOM attributes and downloads the updated HTML file.
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

// Setup Event Listeners
document.addEventListener('DOMContentLoaded', () => {
    // Bind checkbox and text item clicks
    document.querySelectorAll('.check-item').forEach(item => {
        item.addEventListener('click', function(e) {
            // Ignore if clicked on a link or an image
            if (e.target.tagName === 'A' || e.target.tagName === 'IMG') return;
            
            const cb = this.querySelector('input[type="checkbox"]');
            if (cb) {
                // If clicked on the wrapper but not the checkbox directly, toggle it manually
                if (e.target !== cb) cb.checked = !cb.checked;
                styleItem(cb);
                updateProgress();
                saveState();
            } else if (this.classList.contains('text-item') || this.classList.contains('simple-item')) {
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
    document.querySelectorAll('.doc-body img').forEach(img => {
        img.style.cursor = 'zoom-in';
        img.addEventListener('click', function(e) {
            e.stopPropagation();
            openLightbox(this.src);
        });
    });

    // Bind linked fields sync and date shortcut handling (Agent and Date)
    const linkedIds = ['f_info_agent', 'f_sign_agent', 'f_info_date', 'f_sign_date'];
    linkedIds.forEach(id => {
        const el = document.getElementById(id);
        if (el) {
            const isDateField = id.toLowerCase().includes('date');
            const handler = (e) => {
                if (isDateField) {
                    checkDateShortcut(e.target);
                }
                syncLinkedFields(e.target);
                saveState();
            };
            el.addEventListener('input', handler);
            if (isDateField) {
                el.addEventListener('change', handler);
                el.addEventListener('blur', handler);
            }
        }
    });

    // Bind any other date fields present in the document
    document.querySelectorAll('input[id*="date"], input[name*="date"], input.date-field').forEach(input => {
        if (linkedIds.includes(input.id)) return;
        const handler = (e) => {
            checkDateShortcut(e.target);
            saveState();
        };
        input.addEventListener('input', handler);
        input.addEventListener('change', handler);
        input.addEventListener('blur', handler);
    });

    // Bind persistent fields
    document.querySelectorAll('input.persistent-field').forEach(input => {
        input.addEventListener('input', saveState);
    });

    // Initialize state
    loadState();
    syncLinkedFields();
    updateProgress();
});

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
