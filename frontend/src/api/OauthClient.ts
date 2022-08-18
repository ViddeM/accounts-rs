export interface OauthClient {
    id: string,
    clientName: string,
    clientId: string,
    redirectUri: string
}

export interface NewOAuthClient {
    clientId: string,
    clientSecret: string,
}