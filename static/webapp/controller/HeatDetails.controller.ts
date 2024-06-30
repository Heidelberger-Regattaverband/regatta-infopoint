import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import Button, { Button$PressEvent } from "sap/m/Button";
import MessageToast from "sap/m/MessageToast";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class HeatDetailsController extends BaseController {

  formatter: Formatter = Formatter;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);

  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), "heatRegistrations");

    super.getEventBus()?.subscribe("heat", "itemChanged", this.onItemChanged, this);
  }

  private onBeforeShow(): void {
    this.loadHeatModel().then(() => {
      window.addEventListener("keydown", this.keyListener);
    });
  }

  private onBeforeHide(): void {
    window.removeEventListener("keydown", this.keyListener);
  }

  onNavBack(): void {
    const data = (super.getComponentModel("heat") as JSONModel).getData();
    if (data._nav.back) {
      super.navBack(data._nav.back);
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
    this.loadHeatModel().then((updated: boolean) => {
      if (updated) {
        MessageToast.show(this.i18n("msg.dataUpdated"));
      }
    }).finally(() => source.setEnabled(true));
  }

  private async loadHeatModel(): Promise<boolean> {
    const heat: any = (super.getComponentModel("heat") as JSONModel).getData();
    return await super.updateJSONModel(super.getViewModel("heatRegistrations") as JSONModel, `/api/heats/${heat.id}`, super.getView());
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