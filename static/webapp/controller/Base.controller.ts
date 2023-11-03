import Controller from "sap/ui/core/mvc/Controller";
import History from "sap/ui/core/routing/History";
import JSONModel from "sap/ui/model/json/JSONModel";
import EventBus from "sap/ui/core/EventBus";
import Model from "sap/ui/model/Model";
import View from "sap/ui/core/mvc/View";
import Component from "sap/ui/core/Component";
import ResourceBundle from "sap/base/i18n/ResourceBundle";
import ResourceModel from "sap/ui/model/resource/ResourceModel";
import Router from "sap/ui/core/routing/Router";
import Control from "sap/ui/core/Control";
import UIComponent from "sap/ui/core/UIComponent";
import MyComponent from "de/regatta_hd/Component";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class BaseController extends Controller {
  /**
   * Convenience method for accessing the event bus for this component.
   * @returns {sap.ui.core.EventBus} the event bus for this component
   */
  public getEventBus(): EventBus | undefined {
    return super.getOwnerComponent()?.getEventBus();
  }

  /**
   * Convenience method for accessing a component model.
   * @param {string} [name] the model name
   * @returns {sap.ui.model.Model} the model instance
   */
  public getComponentModel(name: string): Model | undefined {
    return super.getOwnerComponent()?.getModel(name);
  }

  /**
   * Convenience method for setting the view model.
   * @param {sap.ui.model.Model} model the model instance
   * @param {string} name the model name
   * @returns {sap.ui.mvc.View} the view instance
   */
  public setComponentModel(model: Model, name: string): Component | undefined {
    return super.getOwnerComponent()?.setModel(model, name);
  }

  /**
   * Convenience method for accessing the router.
   * @returns {sap.ui.core.routing.Router} the router for this component
   */
  public getRouter(): Router {
    const owner = super.getOwnerComponent() as UIComponent;
    return owner?.getRouter();
  }

  /**
   * Convenience method for getting the view model by name.
   * @param {string} [name] the model name
   * @returns {sap.ui.model.Model} the model instance
   */
  public getViewModel(name: string): Model | undefined {
    return this.getView()?.getModel(name);
  }

  /**
   * Convenience method for setting the view model.
   * @param {sap.ui.model.Model} model the model instance
   * @param {string} name the model name
   * @returns {sap.ui.mvc.View} the view instance
   */
  public setViewModel(model: Model, name: string): View | undefined {
    return super.getView()?.setModel(model, name);
  }

  /**
   * Getter for the resource bundle.
   * @returns {sap.base.i18n.ResourceBundle} the resourceModel of the component
   */
  public getResourceBundle(): ResourceBundle | Promise<ResourceBundle> {
    const model: ResourceModel = this.getOwnerComponent()?.getModel("i18n") as ResourceModel;
    return model?.getResourceBundle();
  }

  public navBack(target: string): void {
    const previousHash = History.getInstance().getPreviousHash();
    if (previousHash) {
      window.history.go(-1);
    } else {
      this.getRouter().navTo(target, {}, undefined, true /* no history*/);
    }
  }

  public displayTarget(target: string): void {
    this.getRouter()?.getTargets()?.display(target);
  }

  public i18n(key: string, args?: any[]): string {
    return this.getResourceBundle().getText(key, args);
  }

  public getRegattaId(): int {
    const owner = super.getOwnerComponent() as MyComponent;
    return owner.getRegattaId();
  }

  public async getJSONModel(url: string, control: Control): Promise<JSONModel> {
    const model = new JSONModel();
    await this.updateJSONModel(model, url, control);
    return model;
  }

  public async updateJSONModel(model: JSONModel, url: string, control: Control) {
    if (control) {
      control.setBusy(true);
    }
    await model.loadData(url);
    if (control) {
      control.setBusy(false);
    }
  }
}
