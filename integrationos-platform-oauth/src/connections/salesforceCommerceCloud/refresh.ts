import axios from "axios";
import qs from "qs";
import { DataObject, OAuthResponse } from "../../lib/types";
import { generateBasicHeaders } from "../../lib/helpers";

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
  try {
    const {
      OAUTH_CLIENT_ID: client_id,
      OAUTH_CLIENT_SECRET: client_secret,
      OAUTH_REFRESH_TOKEN: refresh_token,
      OAUTH_REQUEST_PAYLOAD: {
        formData: { SALESFORCE_URL },
      },
      OAUTH_METADATA,
    } = body;

    const requestBody = {
      grant_type: "refresh_token",
      refresh_token,
      client_id,
      client_secret,
    };

    const response = await axios({
      url: `https://${SALESFORCE_URL}/services/oauth2/token`,
      method: "POST",
      headers: generateBasicHeaders(client_id, client_secret),
      data: qs.stringify(requestBody),
    });

    const {
      access_token: accessToken,
      refresh_token: refreshToken,
      id,
      signature,
      instance_url: instanceUrl,
      token_type: tokenType,
    } = response.data;

    return {
      accessToken,
      refreshToken,
      expiresIn: 15 * 60, // Code expires in 15 minutes, converted to seconds
      tokenType,
      meta: {
        ...OAUTH_METADATA?.meta,
        id,
        instanceUrl,
        signature,
      },
    };
  } catch (error) {
    throw new Error(
      `Error fetching refresh token for Salesforce Commerce Cloud: ${error}`
    );
  }
};
