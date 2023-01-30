import { ReactNode } from "react"
import classes from "./Tooltip.module.css";

type TooltipProps = {
  message: string
  children: ReactNode
}

function Tooltip({ message, children }: TooltipProps) {

  return (
    <div className={classes.tooltip}>
      {children}
      <span className={classes.tooltiptext} dangerouslySetInnerHTML={{ __html: message }} />
    </div>
  )
}

export default Tooltip;