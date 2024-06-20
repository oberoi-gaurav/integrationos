import axios from "axios";
import qs from "qs";
import { DataObject, OAuthResponse } from "../../lib/types";

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
  try {
    const {
      clientId: client_id,
      clientSecret: client_secret,
      metadata: {
        additionalData,
        code,
        formData: { SALESFORCE_URL },
        redirectUri,
      },
    } = body;

    const requestBody = {
      grant_type: "authorization_code",
      code,
      client_id,
      client_secret,
      redirect_uri: redirectUri,
    };

    const response = await axios({
      url: `https://${SALESFORCE_URL}/services/oauth2/token`,
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
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
        additionalData,
        id,
        instanceUrl,
        signature,
      },
    };
  } catch (error) {
    throw new Error(
      `Error fetching access token for Salesforce Commerce Cloud: ${error}`
    );
  }
};
