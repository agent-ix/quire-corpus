export function drive(handle: unknown): void {
  (handle as any).upsert();
}
