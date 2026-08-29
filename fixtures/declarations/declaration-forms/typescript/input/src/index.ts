export namespace inner {
  export function nested(): void {}
}

export class Store {
  count = 0;
  upsert(): void {}
}

export interface Persist {
  save(): void;
}

export type Handle = number;

export const freeArrow = (): void => {};

export function freeFunction(): void {}
