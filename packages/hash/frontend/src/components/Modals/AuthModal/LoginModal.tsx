import React, { useCallback, VoidFunctionComponent } from "react";
import { useRouter } from "next/router";
import { ParsedUrlQueryInput } from "querystring";
import { useEffect, useState } from "react";

import { ModalProps } from "../Modal";
import { Layout } from "./Layout";
import { LoginIntro } from "./LoginIntro";
import { VerifyCode } from "./VerifyCode";
import {
  VerificationCodeMetadata,
  LoginWithLoginCodeMutation,
  MutationLoginWithLoginCodeArgs,
  SendLoginCodeMutation,
  SendLoginCodeMutationVariables,
} from "../../../graphql/apiTypes.gen";
import { ApolloError, useMutation } from "@apollo/client";
import {
  sendLoginCode as sendLoginCodeMutation,
  loginWithLoginCode as loginWithLoginCodeMutation,
} from "../../../graphql/queries/user.queries";

enum Screen {
  Intro,
  VerifyCode,
  AccountSetup,
}

type ParsedLoginQuery = {
  verificationId: string;
  verificationCode: string;
};

const isParsedLoginQuery = (
  tbd: ParsedUrlQueryInput
): tbd is ParsedLoginQuery =>
  typeof tbd.verificationId === "string" &&
  typeof tbd.verificationCode === "string";

type LoginModalProps = {
  onLoggedIn?: () => void;
} & Omit<ModalProps, "children">;

const ERROR_CODES = {
  LOGIN_CODE_NOT_FOUND: "An unexpected error occurred, please try again.",
  MAX_ATTEMPTS: "An unexpected error occurred, please try again.",
  INCORRECT: "This login code has expired, please try again.",
} as const;

export const LoginModal: VoidFunctionComponent<LoginModalProps> = ({
  show,
  close,
  onLoggedIn,
}) => {
  // TODO: refactor to use useReducer
  const [activeScreen, setActiveScreen] = useState<Screen>(Screen.Intro);
  const [loginIdentifier, setLoginIdentifier] = useState<string>("");
  const [verificationCode, setVerificationCode] = useState<string>("");
  const [verificationCodeMetadata, setVerificationCodeMetadata] = useState<
    VerificationCodeMetadata | undefined
  >();
  const [errorMessage, setErrorMessage] = useState<string>("");
  const router = useRouter();

  const [sendLoginCodeFn, { loading: sendLoginCodeLoading }] = useMutation<
    SendLoginCodeMutation,
    SendLoginCodeMutationVariables
  >(sendLoginCodeMutation, {
    onCompleted: ({ sendLoginCode }) => {
      setErrorMessage("");
      setVerificationCodeMetadata(sendLoginCode);
      setActiveScreen(Screen.VerifyCode);
    },
    onError: ({ graphQLErrors }) =>
      graphQLErrors.forEach(({ extensions, message }) => {
        const { code } = extensions as { code?: string };
        if (code === "NOT_FOUND") {
          setErrorMessage(message);
        } else {
          throw new ApolloError({ graphQLErrors });
        }
      }),
  });

  const [loginWithLoginCode, { loading: loginWithLoginCodeLoading }] =
    useMutation<LoginWithLoginCodeMutation, MutationLoginWithLoginCodeArgs>(
      loginWithLoginCodeMutation,
      {
        onCompleted: () => {
          if (onLoggedIn) onLoggedIn();
        },
        onError: ({ graphQLErrors }) =>
          graphQLErrors.forEach(({ extensions }) => {
            const { code } = extensions as { code?: keyof typeof ERROR_CODES };

            if (code && Object.keys(ERROR_CODES).includes(code)) {
              reset();
              setErrorMessage(ERROR_CODES[code]);
            } else {
              throw new ApolloError({ graphQLErrors });
            }
          }),
      }
    );

  // handle magic link
  useEffect(() => {
    const { pathname, query } = router;
    if (pathname === "/login" && isParsedLoginQuery(query)) {
      const { verificationId, verificationCode } = query;
      setActiveScreen(Screen.VerifyCode);
      setVerificationCode(verificationCode);
      setTimeout(() => {
        void loginWithLoginCode({
          variables: { verificationId, verificationCode },
        });
      }, 1000);
    }
  }, [router, loginWithLoginCode]);

  const requestLoginCode = (emailOrShortname: string) => {
    let identifier;
    if (emailOrShortname.includes("@")) {
      identifier = emailOrShortname;
    } else {
      identifier = `@${emailOrShortname}`;
    }
    setLoginIdentifier(identifier);
    sendLoginCodeFn({ variables: { emailOrShortname } });
  };

  const login = () => {
    if (!verificationCodeMetadata) return;
    void loginWithLoginCode({
      variables: {
        verificationId: verificationCodeMetadata.id,
        verificationCode,
      },
    });
  };

  const goBack = () => {
    if (activeScreen == Screen.VerifyCode) {
      setActiveScreen(Screen.Intro);
      setErrorMessage("");
      setVerificationCodeMetadata(undefined);
      setVerificationCode("");
    }
  };

  const renderContent = () => {
    switch (activeScreen) {
      case Screen.VerifyCode:
        return (
          <VerifyCode
            loginIdentifier={loginIdentifier}
            loginCode={verificationCode}
            setLoginCode={setVerificationCode}
            goBack={goBack}
            handleSubmit={login}
            loading={loginWithLoginCodeLoading}
            errorMessage={errorMessage}
          />
        );
      case Screen.Intro:
      default:
        return (
          <LoginIntro
            requestLoginCode={requestLoginCode}
            loading={sendLoginCodeLoading}
            errorMessage={errorMessage}
          />
        );
    }
  };

  return (
    <Layout show={show} close={close}>
      {renderContent()}
    </Layout>
  );
};
