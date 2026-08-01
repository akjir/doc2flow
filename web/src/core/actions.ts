import { saveState } from './storage.js';
import { styleItem, updateProgress } from '../features/tasks.js';

export function exportPDF(): void {
    const collapsed = Array.from(document.querySelectorAll<HTMLElement>('.sb.collapsed'));
    collapsed.forEach((el) => el.classList.remove('collapsed'));

    const restore = (): void => {
        collapsed.forEach((el) => el.classList.add('collapsed'));
        window.removeEventListener('afterprint', restore);
    };

    window.addEventListener('afterprint', restore);
    setTimeout(() => window.print(), 100);
}

export function saveDocumentState(): void {
    saveState();

    const checkboxes = document.querySelectorAll<HTMLInputElement>('.check-item input[type="checkbox"]');
    checkboxes.forEach((cb) => {
        if (cb.checked) {
            cb.setAttribute('checked', 'checked');
        } else {
            cb.removeAttribute('checked');
        }
        styleItem(cb);
    });

    const inputs = document.querySelectorAll<HTMLInputElement>('input.persistent-field, .info-table input');
    inputs.forEach((input) => {
        input.setAttribute('value', input.value);
    });

    const rawFilename = window.location.pathname.split('/').pop() ?? 'index.html';
    const filename = decodeURIComponent(rawFilename || 'index.html');

    const htmlContent = '<!DOCTYPE html>\n' + document.documentElement.outerHTML;
    const blob = new Blob([htmlContent], { type: 'text/html;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

export function resetAll(): void {
    const i18n = window.D2F_I18N;
    const confirmMsg = i18n?.confirm_reset ?? 'Are you sure you want to reset all checkboxes?';
    if (!confirm(confirmMsg)) return;

    const checkboxes = document.querySelectorAll<HTMLInputElement>('.check-item input[type="checkbox"]');
    checkboxes.forEach((cb) => {
        cb.checked = false;
        styleItem(cb);
    });
    updateProgress();
    saveState();
}

export function showCopiedFeedback(btn: HTMLElement): void {
    btn.classList.add('copied');
    setTimeout(() => btn.classList.remove('copied'), 2000);
}

export function fallbackCopyText(text: string, btn: HTMLElement): void {
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

export function copyCode(btn: HTMLElement | null): void {
    if (!btn) return;
    const wrap = btn.closest('.code-block-wrap');
    if (!wrap) return;
    const codeEl = wrap.querySelector('code');
    if (!codeEl) return;

    const text = codeEl.innerText || codeEl.textContent || '';
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(() => {
            showCopiedFeedback(btn);
        }).catch(() => fallbackCopyText(text, btn));
    } else {
        fallbackCopyText(text, btn);
    }
}
