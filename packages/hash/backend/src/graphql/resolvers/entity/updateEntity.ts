import { ApolloError } from "apollo-server-express";

import { DbUnknownEntity } from "../../../types/dbTypes";
import {
  MutationUpdateEntityArgs,
  Resolver,
  Visibility,
} from "../../autoGeneratedTypes";
import { GraphQLContext } from "../../context";

export const updateEntity: Resolver<
  Promise<DbUnknownEntity>,
  {},
  GraphQLContext,
  MutationUpdateEntityArgs
> = async (_, { accountId, id, properties }, { dataSources }) => {
  // TODO: doing a select & update for now. See if just update is possible, if not,
  // need to use a transaction

  const entity = await dataSources.db.getEntity({
    accountId: accountId,
    entityId: id,
  });
  if (!entity) {
    throw new ApolloError(
      `Entity ${id} does not exist in account ${accountId}`,
      "NOT_FOUND"
    );
  }

  // Temporary hack - need to figure out how clients side property updates properly. How do they update things on the root entity, e.g. type?
  const propertiesToUpdate = properties.properties ?? properties;

  entity.properties = propertiesToUpdate;

  // TODO: catch error and check if it's a not found
  const updatedEntities = await dataSources.db.updateEntity({
    accountId: accountId,
    entityId: id,
    properties: propertiesToUpdate,
  });

  // TODO: for now, all entities are non-versioned, so the list array only have a single
  // element. Return when versioned entities are implemented at the API layer.
  return {
    ...updatedEntities[0],
    id: updatedEntities[0].entityId,
    accountId: updatedEntities[0].accountId,
    visibility: Visibility.Public, // TODO: get from entity metadata
  } as DbUnknownEntity;
};
