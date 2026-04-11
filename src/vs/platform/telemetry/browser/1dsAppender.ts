/*---------------------------------------------------------------------------------------------
 *  Telemetry stripped — no data is sent.
 *--------------------------------------------------------------------------------------------*/

import { AbstractOneDataSystemAppender, IAppInsightsCore } from '../common/1dsAppender.js';

export class OneDataSystemWebAppender extends AbstractOneDataSystemAppender {
	constructor(
		_isInternalTelemetry: boolean,
		_eventPrefix: string,
		_defaultData: { [key: string]: unknown } | null,
		_iKeyOrClientFactory: string | (() => IAppInsightsCore),
	) {
		super();
	}
}
