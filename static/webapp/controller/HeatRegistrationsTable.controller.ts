import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import MessageToast from "sap/m/MessageToast";
import Button, { Button$PressEvent } from "sap/m/Button";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class HeatRegistrationsTable extends BaseController {

  public formatter: Formatter = Formatter;

  onInit(): void {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());

    super.setViewModel(new JSONModel(), "heatRegistrations");

    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, }, this);

    super.getEventBus()?.subscribe("heat", "itemChanged", this.onItemChanged, this);

    window.addEventListener("keydown", (event: KeyboardEvent) => {
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
          this.onFirstPress();
          break;
        case "ArrowDown":
          this.onLastPress();
          break;
      }
    });
  }

  async onBeforeShow(): Promise<void> {
    await this.loadHeatModel();
  }

  onNavBack(): void {
    const data = (super.getComponentModel("heat") as JSONModel).getData();
    if (data._nav.back) {
      super.displayTarget(data._nav.back);
    } else {
      super.displayTarget("heats");
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

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    await this.loadHeatModel();
    MessageToast.show(this.i18n("msg.dataUpdated", undefined));
    source.setEnabled(true);
  }

  private async loadHeatModel(): Promise<void> {
    const heat: any = (super.getComponentModel("heat") as JSONModel).getData();
    await super.updateJSONModel(super.getViewModel("heatRegistrations") as JSONModel, `/api/heats/${heat.id}`, super.getView());
  }

  private async onItemChanged(channelId: string, eventId: string, parametersMap: any): Promise<void> {
    await this.loadHeatModel();
  }

}