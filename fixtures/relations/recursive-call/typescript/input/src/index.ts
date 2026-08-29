export class Store {
  recurse(n: number): number {
    return n === 0 ? 0 : this.recurse(n - 1);
  }
}

export function plainRecursion(n: number): number {
  return n === 0 ? 0 : plainRecursion(n - 1);
}
