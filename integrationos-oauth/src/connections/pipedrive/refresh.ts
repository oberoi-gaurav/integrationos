import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';
import { generateBasicHeaders } from '../../lib/helpers';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_CLIENT_ID: client_id,
            OAUTH_CLIENT_SECRET: client_secret,
            OAUTH_REFRESH_TOKEN: refresh_token,
        } = body;

        const requestBody = {
            grant_type: 'refresh_token',
            refresh_token,
        };

        const response = await axios.post(
            'https://oauth.pipedrive.com/oauth/token',
            requestBody,
            {
                headers: generateBasicHeaders(client_id, client_secret),
            },
        );

        const {
            access_token: accessToken,
            refresh_token: refreshToken,
            expires_in: expiresIn,
            token_type: tokenType,
        } = response.data;

        return {
            accessToken,
            refreshToken,
            expiresIn,
            tokenType,
            meta: {
                ...body?.OAUTH_METADATA?.meta,
            },
        };
    } catch (error) {
        throw new Error(`Error fetching refresh token for Pipedrive: ${error}`);
    }
};
