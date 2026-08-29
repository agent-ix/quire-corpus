export interface Persist {
  save(): void;
}

export class Runner<T extends Persist> {
  run(): void {}
}

export class Store implements Persist {
  save(): void {}
}
