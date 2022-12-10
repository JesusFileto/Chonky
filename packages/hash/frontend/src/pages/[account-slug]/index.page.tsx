import { useMemo } from "react";
import { Typography, Container, Grid, Skeleton, Box } from "@mui/material";
import { Avatar } from "@hashintel/hash-design-system";
import { getLayoutWithSidebar, NextPageWithLayout } from "../../shared/layout";
import { useOrgs } from "../../components/hooks/useOrgs";
import { useUsers } from "../../components/hooks/useUsers";
import { useRouteAccountInfo } from "../../shared/routing";

const menuBarHeight = 60;

const Page: NextPageWithLayout = () => {
  /**
   * @todo: this will need to be reworked once pages can't rely on the `accountId` being
   * in the URL.
   */
  const { routeAccountSlug: routeAccountIdOrShortname } = useRouteAccountInfo();

  const shortname = routeAccountIdOrShortname.slice(1);

  /**
   * @todo: getting an org or user by their shortname should not be happening
   * client side. This could be addressed by exposing structural querying
   * to the frontend.
   *
   * @see https://app.asana.com/0/1201095311341924/1202863271046362/f
   */
  const { users, loading: loadingUsers } = useUsers();
  const { orgs, loading: loadingOrgs } = useOrgs();

  const profile = useMemo(() => {
    return [...(users ?? []), ...(orgs ?? [])].find(
      (userOrOrg) => userOrOrg.shortname === shortname,
    );
  }, [shortname, users, orgs]);

  const profileNotFound = !profile && !loadingOrgs && !loadingUsers;

  return profileNotFound ? (
    <Container sx={{ paddingTop: 5 }}>
      <Typography variant="h2">Profile not found</Typography>
    </Container>
  ) : (
    <>
      <Box height={menuBarHeight}>{/* @todo: implement the menu-bar */}</Box>
      <Box
        bgcolor={({ palette }) => palette.gray[10]}
        height={`calc(100% - ${menuBarHeight}px)`}
      >
        <Container>
          <Grid container columnSpacing={5} sx={{ marginTop: 0 }}>
            <Grid item md={3}>
              <Box sx={{ position: "relative", top: -15 }}>
                <Avatar
                  bgcolor={
                    profile ? undefined : ({ palette }) => palette.gray[20]
                  }
                  title={
                    profile
                      ? profile.kind === "user"
                        ? profile.preferredName
                        : profile.name
                      : undefined
                  }
                  size={225}
                />
              </Box>
              <Box marginBottom={1}>
                {profile ? (
                  <Typography
                    variant="h1"
                    sx={{ fontSize: 30, fontWeight: 800 }}
                  >
                    {profile.kind === "user"
                      ? profile.preferredName
                      : profile.name}
                  </Typography>
                ) : (
                  <Skeleton height={50} width={200} />
                )}
              </Box>
              {profile ? (
                <Typography
                  sx={{
                    /** @todo: add this color to the MUI theme system */
                    color: "#0775E3",
                    fontSize: 20,
                    fontWeight: 600,
                  }}
                >
                  @{profile.shortname}
                </Typography>
              ) : (
                <Skeleton height={30} width={150} />
              )}
            </Grid>
            <Grid item>{/* @todo: implement the profile page bio */}</Grid>
          </Grid>
        </Container>
      </Box>
    </>
  );
};

Page.getLayout = (page) =>
  getLayoutWithSidebar(page, {
    fullWidth: true,
  });

export default Page;
