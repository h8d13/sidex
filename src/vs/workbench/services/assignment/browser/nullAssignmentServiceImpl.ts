/*---------------------------------------------------------------------------------------------
 *  A/B experimentation stripped — always returns no assignments.
 *--------------------------------------------------------------------------------------------*/

import { InstantiationType, registerSingleton } from '../../../../platform/instantiation/common/extensions.js';
import { IWorkbenchAssignmentService } from '../common/assignmentService.js';
import { NullWorkbenchAssignmentService } from '../test/common/nullAssignmentService.js';

registerSingleton(IWorkbenchAssignmentService, NullWorkbenchAssignmentService, InstantiationType.Delayed);
