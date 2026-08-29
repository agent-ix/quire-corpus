import { Store } from './store';

export function drive(): void {
  const store: Store = new Store();
  store.upsert();
}
