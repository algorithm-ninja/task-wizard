import { Injectable } from '@angular/core';
import { Query } from 'apollo-angular';
import gql from 'graphql-tag';
import { SubmissionListQuery, SubmissionListQueryVariables } from './__generated__/SubmissionListQuery';

@Injectable({
  providedIn: 'root'
})
export class SubmissionListQueryService extends Query<SubmissionListQuery, SubmissionListQueryVariables> {
  document = gql`
    query SubmissionListQuery($userId: String!) {
      problems {
        name
        submissions(userId: $userId) {
          id
          createdAt
          files {
            fieldId
            typeId
            name
            contentBase64
          }
          status
          scorables {
            scorableId
            score
          }
        }
      }
    }
  `;
}