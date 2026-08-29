import { Store } from './store';
import { Cache } from './cache';

export function drive(handle: any): void {
  handle.upsert();
}
