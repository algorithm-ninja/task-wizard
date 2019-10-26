import { Component, Input, OnInit } from '@angular/core';
import { AppComponent } from '../app.component';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { SubmissionListQueryService } from '../submission-list-query.service';
import { QueryRef } from 'apollo-angular';
import { SubmissionListQuery, SubmissionListQueryVariables, SubmissionListQuery_problems, SubmissionListQuery_problems_submissions } from '../__generated__/SubmissionListQuery';
import { ContestQuery_problems_material, ContestQuery_problems_material_scorables } from '../__generated__/ContestQuery';
import { SubmissionDialogComponent } from '../submission-dialog/submission-dialog.component';
import { SubmissionQueryService } from '../submission-query.service';

@Component({
  selector: 'app-submission-list-dialog',
  templateUrl: './submission-list-dialog.component.html',
  styleUrls: ['./submission-list-dialog.component.scss']
})
export class SubmissionListDialogComponent implements OnInit {

  constructor(
    private modal: NgbModal,
    private submissionListQueryService: SubmissionListQueryService,
    private submissionQueryService: SubmissionQueryService,
  ) { }

  @Input()
  appComponent: AppComponent;

  @Input()
  problemName: string;

  submissionListQuery: QueryRef<SubmissionListQuery, SubmissionListQueryVariables>;

  ngOnInit() {
    this.submissionListQuery = this.submissionListQueryService.watch({
      userId: 'test', // FIXME
    });
  }

  getState(scorable: ContestQuery_problems_material_scorables, submission: SubmissionListQuery_problems_submissions) {
    return submission.scorables.find((s) => s.scorableId === scorable.name) || { score: 0 };
  }

  async openDetail(submission: SubmissionListQuery_problems_submissions) {
    const modalRef = this.modal.open(SubmissionDialogComponent);
    const modal = modalRef.componentInstance;

    modal.appComponent = this.appComponent;
    modal.problemName = this.problemName;
    modal.setSubmissionQueryRef(this.submissionQueryService.watch({
      submissionId: submission.id,
    }));

    try {
      await modalRef.result;
    } catch (e) {
      // No-op
    }
  }

}
