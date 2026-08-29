import { Store } from './store';

export function drive(handle: any): void {
  handle.upsert();
}
