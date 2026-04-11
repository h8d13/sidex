/*---------------------------------------------------------------------------------------------
 *  Telemetry stripped — no data is sent.
 *--------------------------------------------------------------------------------------------*/

import { AbstractOneDataSystemAppender, IAppInsightsCore } from '../common/1dsAppender.js';
import { IRequestService } from '../../request/common/request.js';

export class OneDataSystemAppender extends AbstractOneDataSystemAppender {
	constructor(
		_requestService: IRequestService | undefined,
		_isInternalTelemetry: boolean,
		_eventPrefix: string,
		_defaultData: { [key: string]: unknown } | null,
		_iKeyOrClientFactory: string | (() => IAppInsightsCore),
	) {
		super();
	}
}
