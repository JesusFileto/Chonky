import { Entity } from "../autoGeneratedTypes";

import { entityNamespaceName } from "./shared/namespace";
import {
  aggregateEntity,
  createEntity,
  entity,
  entityFields,
  updateEntity,
} from "./entity";
import { blockFields } from "./block";
import {
  createPage,
  insertBlockIntoPage,
  namespacePages,
  page,
  updatePage,
} from "./pages";
import { namespaces } from "./namespace/namespaces";

import { DbOrg, DbUser } from "../../types/dbTypes";

const KNOWN_ENTITIES = ["Page", "Text", "User"];

export const resolvers = {
  Query: {
    aggregateEntity,
    entity,
    namespacePages,
    namespaces,
    page,
  },

  Mutation: {
    createEntity,
    createPage,
    insertBlockIntoPage,
    updateEntity,
    updatePage,
  },

  Block: {
    namespace: entityNamespaceName,
  },

  BlockProperties: {
    entity: blockFields.entity,
  },

  UnknownEntity: {
    properties: entityFields.properties,
  },

  Entity: {
    __resolveType(entity: Entity) {
      if (KNOWN_ENTITIES.includes(entity.type)) {
        return entity.type;
      }
      return "UnknownEntity";
    },
  },

  Namespace: {
    __resolveType(entity: DbUser | DbOrg) {
      return entity.type;
    },
  },
};
