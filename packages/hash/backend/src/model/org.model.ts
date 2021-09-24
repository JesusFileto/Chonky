import {
  Org,
  Account,
  AccountConstructorArgs,
  OrgInvitation,
  OrgEmailInvitation,
} from ".";
import { DBClient } from "../db";
import { DBOrgProperties, EntityType } from "../db/adapter";
import { genId } from "../util";

type OrgConstructorArgs = {
  properties: DBOrgProperties;
} & Omit<AccountConstructorArgs, "type">;

class __Org extends Account {
  properties: DBOrgProperties;

  constructor({ properties, ...remainingArgs }: OrgConstructorArgs) {
    super({ ...remainingArgs, properties });
    this.properties = properties;
  }

  static getEntityType = async (client: DBClient): Promise<EntityType> =>
    client
      .getSystemTypeLatestVersion({ systemTypeName: "Org" })
      .then((OrgEntityType) => {
        if (!OrgEntityType) {
          throw new Error("Org system entity type not found in datastore");
        }

        return OrgEntityType;
      });

  static getOrgById =
    (client: DBClient) =>
    ({
      accountId,
      entityId,
    }: {
      accountId: string;
      entityId: string;
    }): Promise<Org | null> =>
      client
        .getEntityLatestVersion({ accountId, entityId })
        .then((dbOrg) => (dbOrg ? new Org(dbOrg) : null));

  static getOrgByShortname =
    (client: DBClient) =>
    ({ shortname }: { shortname: string }): Promise<Org | null> =>
      client
        .getOrgByShortname({ shortname })
        .then((dbUser) => (dbUser ? new Org(dbUser) : null));

  static createOrg =
    (client: DBClient) =>
    async (params: {
      createdById: string;
      properties: DBOrgProperties;
    }): Promise<Org> => {
      const accountId = genId();

      const entity = await client.createEntity({
        ...params,
        accountId,
        entityTypeId: (await Org.getEntityType(client)).entityId,
        versioned: false, // @todo: should Org's be versioned?
      });

      const org = new Org(entity);

      await OrgInvitation.createOrgInvitation(client)({ org });

      return org;
    };

  private updateOrgProperties =
    (client: DBClient) => (properties: DBOrgProperties) =>
      this.updateProperties(client)(properties);

  /**
   * @returns all email invitations associated with the organization.
   */
  getEmailInvitations = async (
    client: DBClient
  ): Promise<OrgEmailInvitation[]> => {
    /** @todo: query for email invitations with correct outgoing 'org' relationships */
    const dbEntities = await client.getEntitiesBySystemType({
      accountId: this.accountId,
      systemTypeName: "OrgEmailInvitation",
    });

    return dbEntities.map((entity) => new OrgEmailInvitation(entity));
  };

  /**
   * @returns the email invitation associated with the organization that has a matching token, or null.
   */
  getEmailInvitationWithToken =
    (client: DBClient) =>
    async (
      emailInvitationToken: string
    ): Promise<OrgEmailInvitation | null> => {
      const emailInvitations = await this.getEmailInvitations(client);

      return (
        emailInvitations.find(
          ({ properties }) => properties.accessToken === emailInvitationToken
        ) || null
      );
    };
}

export default __Org;
