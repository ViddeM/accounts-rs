export interface OauthClient {
    clientName: string,
    clientId: string,
    redirectUri: string
}

export interface NewOAuthClient {
    clientId: string,
    clientSecret: string,
}