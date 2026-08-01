import { registerSaveHandler, registerLoadHandler } from './storage.js';
import { registerResetHandler } from './core.js';

export interface CommentBoxResult {
    readonly box: HTMLElement;
    readonly input: HTMLTextAreaElement;
}

export function autoExpandTextarea(el: HTMLTextAreaElement | null): void {
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = String(el.scrollHeight) + 'px';
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

export function saveComments(): Record<string, unknown> {
    const comments: Record<string, string> = {};
    document.querySelectorAll<HTMLElement>('.check-item').forEach((item, index) => {
        const input = item.querySelector<HTMLTextAreaElement>('.item-comment-input');
        if (input && input.value.trim() !== '') {
            const key = item.id || ('item_' + String(index));
            comments[key] = input.value;
        }
    });
    return { comments };
}

export function loadComments(state: Record<string, unknown>): boolean {
    const comments = state['comments'];
    if (typeof comments === 'object' && comments !== null && !Array.isArray(comments)) {
        const commentsRecord = comments as Record<string, string>;
        document.querySelectorAll<HTMLElement>('.check-item').forEach((item, index) => {
            const key = item.id || ('item_' + String(index));
            const val = commentsRecord[key];
            if (val !== undefined && typeof val === 'string') {
                getOrCreateCommentBox(item, val);
            }
        });
    }
    return false;
}

export function resetComments(): void {
    document.querySelectorAll<HTMLElement>('.item-comment-box').forEach((box) => {
        box.remove();
    });
}

registerSaveHandler(saveComments);
registerLoadHandler(loadComments);
registerResetHandler(resetComments);
