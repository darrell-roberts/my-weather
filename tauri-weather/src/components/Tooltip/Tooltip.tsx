import { ReactNode } from "react"
import classes from "./Tooltip.module.css";

type TooltipProps = {
  message?: string;
  htmlMessage?: string;
  children: ReactNode;
}

function Tooltip({ message, children, htmlMessage }: TooltipProps) {

  return (
    <div className={classes.tooltip}>
      {children}
      {htmlMessage &&
        <span className={classes.tooltiptext} dangerouslySetInnerHTML={{ __html: htmlMessage }} />
      }
      {
        message &&
        <span className={classes.tooltiptext}>{message}</span>
      }
    </div>
  )
}

export default Tooltip;