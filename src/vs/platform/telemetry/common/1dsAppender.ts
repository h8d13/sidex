/*---------------------------------------------------------------------------------------------
 *  Telemetry stripped — no data is sent.
 *--------------------------------------------------------------------------------------------*/

import { ITelemetryAppender } from './telemetryUtils.js';

export interface IAppInsightsCore {
	pluginVersionString: string;
	track(item: unknown): void;
	unload(isAsync: boolean, unloadComplete: (unloadState: unknown) => void): void;
}

export abstract class AbstractOneDataSystemAppender implements ITelemetryAppender {
	protected readonly endPointUrl = '';
	protected readonly endPointHealthUrl = '';
	protected _aiCoreOrKey: IAppInsightsCore | string | undefined = undefined;
	log(_eventName: string, _data?: unknown): void { }
	flush(): Promise<void> { return Promise.resolve(); }
}
