export interface Persist {
  save(): void;
}

export class Base {
  common(): void {}
}

export class Store extends Base implements Persist {
  save(): void {}
}
