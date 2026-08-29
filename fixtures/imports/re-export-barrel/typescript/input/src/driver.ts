import { Store } from './barrel';

export function drive(store: Store): void {
  store.upsert();
}
