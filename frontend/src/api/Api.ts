import axios, { AxiosResponse } from "axios";
import encodeurl from "encodeurl";
import { Me } from "./Me";
import { User } from "./User";
import { NewOAuthClient, OauthClient } from "./OauthClient";

let baseUrl = "/api";
// Check if we're server side
if (typeof window === "undefined") {
  baseUrl = process.env.NEXT_PUBLIC_BASE_URL || "/api";
}

axios.defaults.baseURL = baseUrl;

axios.interceptors.request.use(
  (config) => {
    // Do something before request is sent
    config.url = encodeurl(config.url ?? "");
    return config;
  },
  (error) => {
    // Do something with request error
    return Promise.reject(error);
  }
);

interface RawApiResponse<ResponseData> {
  success?: ResponseData;
  error_msg?: string;
}

export const Api = {
  me: {
    getMe: (cookie?: string) => {
      return get<Me>("/me", cookie);
    },
    postLogout: () => {
      return handleResponse(axios.post<RawApiResponse<void>>("/logout", {}));
    },
  },
  user: {
    getAll: (cookie?: string) => {
      return get<{ users: User[] }>("/users", cookie);
    },
  },
  whitelist: {
    getAll: (cookie?: string) => {
      return get<{ emails: String[] }>("/whitelist", cookie);
    },
    addToWhitelist: (email: string) => {
      return handleResponse(
        axios.post<RawApiResponse<{}>>("/whitelist", {
          email: email,
        })
      );
    },
    removeFromWhitelist: (email: string) => {
      return handleResponse(
        axios.delete<RawApiResponse<{}>>(`/whitelist/${email}`)
      );
    },
  },
  oauthClients: {
    getAll: (cookie?: string) => {
      return get<{ oauthClients: OauthClient[] }>("/oauth_clients", cookie);
    },
    create: (clientName: string, redirectUri: string) => {
      return handleResponse(
        axios.post<RawApiResponse<NewOAuthClient>>("/oauth_clients", {
          clientName: clientName,
          redirectUri: redirectUri,
        })
      );
    },
    remove: (id: string) => {
      return handleResponse(
        axios.delete<RawApiResponse<{}>>(`/oauth_clients/${id}`)
      );
    },
  },
};

function get<T>(endpoint: string, cookie?: string): Promise<ApiResponse<T>> {
  return handleResponse(
    axios.get<RawApiResponse<T>>(endpoint, {
      headers: cookie ? { cookie: cookie } : undefined,
      withCredentials: true,
    })
  );
}

export interface ApiResponse<T> {
  failedToReachBackend?: boolean;
  isError?: boolean;
  redirect?: string;
  data?: T;
  rawResponse: AxiosResponse<RawApiResponse<T>>;
}

function handleResponse<T>(
  response: Promise<AxiosResponse<RawApiResponse<T>, any>>
): Promise<ApiResponse<T>> {
  return response
    .then((res) => {
      return {
        data: res.data?.success ?? undefined,
        rawResponse: res,
      };
    })
    .catch((err) => {
      if (err.errno === -111) {
        // Failed to reach the server
        return {
          isError: true,
          failedToReachBackend: true,
          rawResponse: err.respone,
        };
      }

      if (err.response?.status === 401) {
        return {
          redirect: err.response.headers.location,
          rawResponse: err.response,
        };
      }
      console.error("ERROR!!! ", err);
      // TODO: Implement handling of error message

      return {
        isError: true,
        rawResponse: err.response,
      };
    });
}

export interface ApiResponse<T> {
  errorTranslationString?: string;
  error?: boolean;
  data?: T;
  rawResponse: AxiosResponse<RawApiResponse<T>>;
}

export function isClientSide(): boolean {
  return typeof document !== "undefined";
}
