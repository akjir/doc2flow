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
