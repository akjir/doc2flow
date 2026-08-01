import { saveState } from '../core/storage.js';

export function styleItem(cb: HTMLInputElement): void {
    const item = cb.closest<HTMLElement>('.check-item');
    if (item) {
        item.classList.toggle('checked', cb.checked);
    }
}

export function autoExpandTextarea(el: HTMLTextAreaElement | null): void {
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = String(el.scrollHeight) + 'px';
}

export interface CommentBoxResult {
    readonly box: HTMLElement;
    readonly input: HTMLTextAreaElement;
}

export function getOrCreateCommentBox(checkItem: HTMLElement | null, initialValue?: string): CommentBoxResult | null {
    if (!checkItem) return null;
    let box = checkItem.querySelector<HTMLElement>('.item-comment-box');
    let input: HTMLTextAreaElement | null = null;

    if (!box) {
        box = document.createElement('div');
        box.className = 'item-comment-box';

        input = document.createElement('textarea');
        input.rows = 1;
        input.className = 'item-comment-input';
        const i18n = window.D2F_I18N ?? {};
        const commentLabel = i18n.comment_placeholder ?? 'Add a comment...';
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
        const rawInput = box.querySelector('.item-comment-input');
        input = rawInput instanceof HTMLTextAreaElement ? rawInput : null;
    }

    if (!input) return null;

    if (typeof initialValue === 'string') {
        input.value = initialValue;
        input.textContent = initialValue;
        input.setAttribute('value', initialValue);
    }

    autoExpandTextarea(input);
    return { box, input };
}

export function updateProgress(): void {
    const i18n = window.D2F_I18N ?? {};
    const sections = document.querySelectorAll<HTMLElement>('.section');
    let total = 0;
    let done = 0;

    const updates = Array.from(sections).map((sec) => {
        const cbs = Array.from(sec.querySelectorAll<HTMLInputElement>('input[type="checkbox"]'));
        const badge = sec.querySelector<HTMLElement>('.sbadge');
        const count = cbs.length;
        const checkedCount = cbs.filter((c) => c.checked).length;
        total += count;
        done += checkedCount;
        return { badge, count, checkedCount };
    });

    updates.forEach(({ badge, count, checkedCount }) => {
        if (!badge) return;
        if (count === 0) {
            badge.textContent = '';
            badge.style.display = 'none';
        } else {
            badge.style.display = '';
            badge.textContent = String(checkedCount) + ' / ' + String(count);
            badge.className = 'sbadge' + (checkedCount === count ? ' done' : '');
        }
    });

    const pct = total ? Math.round((done / total) * 100) : 0;
    const pb = document.getElementById('pb');
    if (pb) {
        pb.style.width = String(pct) + '%';
        if (pb.parentElement) {
            pb.parentElement.setAttribute('aria-valuenow', String(pct));
        }
    }

    const pt = document.getElementById('pt');
    if (pt) {
        const tmpl = i18n.progress_template ?? '{done} of {total} tasks completed ({pct}%)';
        pt.textContent = tmpl
            .replace('{done}', String(done))
            .replace('{total}', String(total))
            .replace('{pct}', String(pct));
    }

    const finishBox = document.getElementById('finish-box');
    const finishIcon = document.getElementById('finish-icon');
    const finishTitle = document.getElementById('finish-title');
    const rawPdf = document.getElementById('btn-pdf');
    const btnPdf = rawPdf instanceof HTMLButtonElement ? rawPdf : null;

    if (finishBox) {
        finishBox.classList.remove('completed', 'pending', 'no-tasks');
        if (total === 0) {
            finishBox.classList.add('no-tasks');
            if (btnPdf) btnPdf.disabled = false;
        } else if (done < total) {
            finishBox.classList.add('pending');
            if (finishIcon) finishIcon.innerHTML = '&#x29D6;';
            if (finishTitle) finishTitle.textContent = i18n.setup_in_progress ?? 'Setup in Progress';
            if (btnPdf) btnPdf.disabled = true;
        } else {
            finishBox.classList.add('completed');
            if (finishIcon) finishIcon.innerHTML = '&#x2714;';
            if (finishTitle) finishTitle.textContent = i18n.setup_completed ?? 'Setup Completed';
            if (btnPdf) btnPdf.disabled = false;
        }
    }
}

if (typeof window !== 'undefined') {
    document.addEventListener('DOMContentLoaded', () => {
        updateProgress();
    });
}
