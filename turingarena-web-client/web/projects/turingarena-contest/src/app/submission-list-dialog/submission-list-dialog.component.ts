import { Component, Input } from '@angular/core';
import { NgbActiveModal, NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { Apollo } from 'apollo-angular';

import { ProblemFragment } from '../fragments/__generated__/ProblemFragment';

@Component({
  selector: 'app-submission-list-dialog',
  templateUrl: './submission-list-dialog.component.html',
  styleUrls: ['./submission-list-dialog.component.scss'],
})
export class SubmissionListDialogComponent  {

  constructor(
    private readonly apollo: Apollo,
    readonly modalService: NgbModal,
  ) { }

  @Input()
  modal!: NgbActiveModal;

  @Input()
  problem!: ProblemFragment;

}