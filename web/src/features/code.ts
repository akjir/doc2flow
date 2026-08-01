declare global {
    interface Window {
        copyCode?: (btn: HTMLElement | null) => Promise<void>;
    }
}

// WeakMap stores the active timer per button to prevent race conditions
const feedbackTimers = new WeakMap<HTMLElement, number>();

export function showCopiedFeedback(btn: HTMLElement): void {
    btn.classList.add('copied');

    // If a timer is already running for this button, clear it
    if (feedbackTimers.has(btn)) {
        window.clearTimeout(feedbackTimers.get(btn));
    }

    const timer = window.setTimeout(() => {
        btn.classList.remove('copied');
        feedbackTimers.delete(btn);
    }, 2000);

    feedbackTimers.set(btn, timer);
}

export function fallbackCopyText(text: string, btn: HTMLElement): boolean {
    const ta = document.createElement('textarea');
    ta.value = text;
    // Move out of viewport instead of opacity: 0 (more robust on mobile devices)
    ta.style.position = 'fixed';
    ta.style.left = '-9999px';
    ta.style.top = '0';
    ta.setAttribute('readonly', ''); // Prevents the mobile keyboard from popping up

    document.body.appendChild(ta);
    ta.select();
    ta.setSelectionRange(0, text.length);

    let success = false;
    try {
        success = document.execCommand('copy');
        if (success) {
            showCopiedFeedback(btn);
        }
    } catch (e) {
        console.error('Fallback copy failed', e);
    } finally {
        document.body.removeChild(ta);
    }

    return success;
}

export async function copyCode(btn: HTMLElement | null): Promise<void> {
    if (!btn) return;

    const wrap = btn.closest('.code-block-wrap');
    if (!wrap) return;

    const codeEl = wrap.querySelector('code');
    if (!codeEl) return;

    const text = codeEl.textContent ?? '';

    // Use modern Async Clipboard API when available
    if (navigator.clipboard && window.isSecureContext) {
        try {
            await navigator.clipboard.writeText(text);
            showCopiedFeedback(btn);
            return;
        } catch (err) {
            console.warn('Clipboard API failed, falling back to execCommand:', err);
        }
    }

    // Execute fallback if Clipboard API is unavailable or fails
    fallbackCopyText(text, btn);
}

if (typeof window !== 'undefined') {
    window.copyCode = copyCode;
}
 