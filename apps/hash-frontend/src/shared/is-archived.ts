import { types } from "@local/hash-isomorphic-utils/ontology-types";
import {
  DataTypeWithMetadata,
  Entity,
  EntityTypeWithMetadata,
  PropertyTypeWithMetadata,
} from "@local/hash-subgraph";
import { extractBaseUrl } from "@local/hash-subgraph/type-system-patch";

import { isEntityPageEntity, isType } from "./is-of-type";

export const isTypeArchived = (
  type:
    | EntityTypeWithMetadata
    | PropertyTypeWithMetadata
    | DataTypeWithMetadata,
) =>
  type.metadata.custom.temporalVersioning.transactionTime.end.kind ===
  "exclusive";

export const isItemArchived = (
  item:
    | Entity
    | EntityTypeWithMetadata
    | PropertyTypeWithMetadata
    | DataTypeWithMetadata,
) => {
  if (isType(item)) {
    return isTypeArchived(item);
  } else if (isEntityPageEntity(item)) {
    return item.properties[
      extractBaseUrl(types.propertyType.archived.propertyTypeId)
    ] as boolean;
  }
  return false;
};