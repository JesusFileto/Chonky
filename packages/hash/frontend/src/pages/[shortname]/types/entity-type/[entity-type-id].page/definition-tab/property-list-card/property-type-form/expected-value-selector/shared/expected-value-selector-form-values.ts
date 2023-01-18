import { BaseUri } from "@blockprotocol/type-system";

import { FlattenedCustomExpectedValueList } from "../../../shared/expected-value-types";

export type ExpectedValueSelectorFormValues = {
  propertyTypeBaseUri?: BaseUri;
  customExpectedValueId?: string;
  editingExpectedValueIndex?: number;
  flattenedCustomExpectedValueList: FlattenedCustomExpectedValueList;
};
