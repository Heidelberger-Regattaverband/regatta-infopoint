import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import HeatsTableController from "./HeatsTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class HeatDetailsController extends BaseController {

  private static readonly ENTRIES_MODEL: string = "heatEntries";

  readonly formatter: Formatter = Formatter;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);
  private heatId?: number;

  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), HeatDetailsController.ENTRIES_MODEL);

    super.getRouter()?.getRoute("heatDetails")?.attachPatternMatched(
      (event: Route$PatternMatchedEvent) => this.onPatternMatched(event), this);
  }

  private onPatternMatched(event: Route$PatternMatchedEvent): void {
    this.heatId = (event.getParameter("arguments") as any)?.heatId;
  }

  private onBeforeShow(): void {
    this.loadHeatModel().then(() => {
      super.getEventBus()?.subscribe("heat", "itemChanged", this.onItemChanged, this);
      window.addEventListener("keydown", this.keyListener);
    });
  }

  private onBeforeHide(): void {
    window.removeEventListener("keydown", this.keyListener);
    super.getEventBus()?.unsubscribe("heat", "itemChanged", this.onItemChanged, this);
  }

  onNavBack(): void {
    const heat: any = (super.getComponentModel(HeatsTableController.HEAT_MODEL) as JSONModel).getData();
    if (heat._nav?.back) {
      super.navBack(heat._nav.back);
    } else {
      super.navBack("heats");
    }
  }

  onFirstPress(): void {
    super.getEventBus()?.publish("heat", "first", {});
  }

  onPreviousPress(): void {
    super.getEventBus()?.publish("heat", "previous", {});
  }

  onNextPress(): void {
    super.getEventBus()?.publish("heat", "next", {});
  }

  onLastPress(): void {
    super.getEventBus()?.publish("heat", "last", {});
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadHeatModel().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => source.setEnabled(true));
  }

  private async loadHeatModel(): Promise<boolean> {
    const heat: any = (super.getComponentModel(HeatsTableController.HEAT_MODEL) as JSONModel).getData();
    if (heat?.id) {
      this.heatId = heat.id;
    };
    const url: string = `/api/heats/${this.heatId}`;
    return await super.updateJSONModel(super.getViewModel(HeatDetailsController.ENTRIES_MODEL) as JSONModel, url, super.getView());
  }

  private async onItemChanged(channelId: string, eventId: string, parametersMap: any): Promise<void> {
    await this.loadHeatModel();
  }

  private onKeyDown(event: KeyboardEvent): void {
    switch (event.key) {
      case "F5":
        event.preventDefault();
        break;
      case "ArrowLeft":
        this.onPreviousPress();
        break;
      case "ArrowRight":
        this.onNextPress();
        break;
      case "ArrowUp":
      case "Home":
        this.onFirstPress();
        break;
      case "ArrowDown":
      case "End":
        this.onLastPress();
        break;
    }
  }

}