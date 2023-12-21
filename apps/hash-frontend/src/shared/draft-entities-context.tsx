import { useQuery } from "@apollo/client";
import {
  currentTimeInstantTemporalAxes,
  mapGqlSubgraphFieldsFragmentToSubgraph,
  zeroedGraphResolveDepths,
} from "@local/hash-isomorphic-utils/graph-queries";
import { Entity, EntityRootType, Subgraph } from "@local/hash-subgraph/.";
import { getRoots } from "@local/hash-subgraph/stdlib";
import {
  createContext,
  FunctionComponent,
  PropsWithChildren,
  useContext,
  useMemo,
  useState,
} from "react";

import {
  StructuralQueryEntitiesQuery,
  StructuralQueryEntitiesQueryVariables,
} from "../graphql/api-types.gen";
import { structuralQueryEntitiesQuery } from "../graphql/queries/knowledge/entity.queries";

export type DraftEntitiesContextValue = {
  draftEntities?: Entity[];
  draftEntitiesSubgraph?: Subgraph<EntityRootType>;
  loading: boolean;
  refetch: () => Promise<void>;
};

export const DraftEntitiesContext =
  createContext<null | DraftEntitiesContextValue>(null);

export const useDraftEntities = () => {
  const draftEntitiesContext = useContext(DraftEntitiesContext);

  if (!draftEntitiesContext) {
    throw new Error("Context missing");
  }

  return draftEntitiesContext;
};

const draftEntitiesPollingInterval = 10_000;

export const DraftEntitiesContextProvider: FunctionComponent<
  PropsWithChildren
> = ({ children }) => {
  const [
    previouslyFetchedDraftEntitiesData,
    setPreviouslyFetchedDraftEntitiesData,
  ] = useState<StructuralQueryEntitiesQuery>();

  const {
    data: draftEntitiesData,
    refetch,
    loading,
  } = useQuery<
    StructuralQueryEntitiesQuery,
    StructuralQueryEntitiesQueryVariables
  >(structuralQueryEntitiesQuery, {
    variables: {
      query: {
        filter: {
          all: [
            {
              equal: [{ path: ["draft"] }, { parameter: true }],
            },
            {
              equal: [{ path: ["archived"] }, { parameter: false }],
            },
          ],
        },
        temporalAxes: currentTimeInstantTemporalAxes,
        graphResolveDepths: zeroedGraphResolveDepths,
        includeDrafts: true,
      },
      includePermissions: false,
    },
    onCompleted: (data) => setPreviouslyFetchedDraftEntitiesData(data),
    pollInterval: draftEntitiesPollingInterval,
    fetchPolicy: "network-only",
  });

  const draftEntitiesSubgraph = useMemo(
    () =>
      draftEntitiesData || previouslyFetchedDraftEntitiesData
        ? mapGqlSubgraphFieldsFragmentToSubgraph<EntityRootType>(
            (draftEntitiesData ?? previouslyFetchedDraftEntitiesData)!
              .structuralQueryEntities.subgraph,
          )
        : undefined,
    [draftEntitiesData, previouslyFetchedDraftEntitiesData],
  );

  const draftEntities = useMemo(
    () => (draftEntitiesSubgraph ? getRoots(draftEntitiesSubgraph) : undefined),
    [draftEntitiesSubgraph],
  );

  const value = useMemo<DraftEntitiesContextValue>(
    () => ({
      draftEntities,
      draftEntitiesSubgraph,
      loading,
      refetch: async () => {
        await refetch();
      },
    }),
    [draftEntities, draftEntitiesSubgraph, loading, refetch],
  );

  return (
    <DraftEntitiesContext.Provider value={value}>
      {children}
    </DraftEntitiesContext.Provider>
  );
};