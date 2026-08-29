export interface Persist {
  save(): number;
}

export class Store implements Persist {
  public exported(id: number): number {
    return id;
  }

  protected crateVisible(): void {}

  private privateHelper(): void {}

  save(): number {
    return 0;
  }
}

class NotExported {
  method(): void {}
}

export const keep = new NotExported();
