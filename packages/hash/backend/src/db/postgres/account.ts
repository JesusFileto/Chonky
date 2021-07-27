import { Connection } from "./types";

import { sql } from "slonik";

export const insertAccount = async (
  conn: Connection,
  params: { accountId: string }
): Promise<void> => {
  await conn.query(sql`
    insert into accounts (account_id) values (${params.accountId})
    on conflict (account_id) do nothing`);
};

export const insertEntityAccount = async (
  conn: Connection,
  params: { entityId: string; accountId: string }
): Promise<void> => {
  await conn.query(sql`
    insert into entity_account (entity_id, account_id)
    values (${params.entityId}, ${params.accountId})`);
};

export const getEntityAccount = async (
  conn: Connection,
  entityId: string
): Promise<string | null> => {
  const row = await conn.maybeOne(sql`
    select account_id from entity_account where entity_id = ${entityId}`);
  return row ? (row["account_id"] as string) : null;
};
