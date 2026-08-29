import { Store } from './barrel';

export function drive(handle: any): void {
  handle.upsert();
}
