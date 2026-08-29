import { Store } from './store';

export function drive(store: Store): void {
  store.upsert();
}
