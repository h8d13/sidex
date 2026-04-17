import { createDecorator } from '../../../../platform/instantiation/common/instantiation.js';

export const IWorkbenchIssueService = createDecorator<IWorkbenchIssueService>('workbenchIssueService');

export interface IWorkbenchIssueService {
	readonly _serviceBrand: undefined;
	openReporter(): Promise<void>;
}
