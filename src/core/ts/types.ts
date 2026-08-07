import type { Core } from './core.ts';
import type { Export } from './export.ts';
import type { Language } from './lang.ts';
import type { Storage } from './storage.ts';
import type { Utils } from './utils.ts';

declare global {
    interface Window {
        d2f: {
            core: Core;
            document: {
                id: String;
            };
            storage: Storage;
            export: Export;
            utils: Utils;
            lang: Language;
        }
    }
}