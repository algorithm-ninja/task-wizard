import { css, cx } from 'emotion';
import React, { useEffect, useState } from 'react';
import { useAsync } from '../async-hook';
import { animationFrame, delay } from '../delay';

interface Props {
  onClose: () => void;
  show: boolean;
  children: React.ReactElement;
}

export const modalHeaderCss = 'modal-header';
export const modalBodyCss = 'modal-body';
export const modalFooterCss = 'modal-footer';

export function Modal({ onClose, show = true, children }: Props) {
  const [show1, setShow1] = useState(false);
  const [show2, setShow2] = useState(false);

  const [animate, { loading: animating }] = useAsync(async () => {
    if (show) {
      setShow1(show);
      await animationFrame();
      setShow2(show);
    } else {
      setShow2(show);
      await delay(300);
      await animationFrame();
      setShow1(show);
    }
  });

  useEffect(() => {
    if (!animating) {
      animate().catch(e => console.error(e));
    }
  }, [show]);

  return (
    <>
      <div
        className={cx(
          'modal',
          'fade',
          ...(show1
            ? [
                css`
                  display: block;
                  pointer-events: none;
                `,
              ]
            : []),
          ...(show2 ? ['show'] : []),
        )}
      >
        <div className={cx('modal-dialog')}>
          <div className={cx('modal-content')}>{children}</div>
        </div>
      </div>
      {show1 && (
        <div
          className={cx(
            'modal-backdrop',
            'fade',
            ...(show2 ? ['show'] : []),
            // css`
            //   z-index: 1051;
            // `,
          )}
          onClick={onClose}
        ></div>
      )}
    </>
  );
}
