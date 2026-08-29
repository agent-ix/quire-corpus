import { StoreView } from '../ui/store';

// TC-001, FR-001-AC-1: the view renders the stored record.
test('renders', () => {
  expect(new StoreView().render()).toBe('store');
});
